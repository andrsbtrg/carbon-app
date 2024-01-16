use std::{collections::HashSet, str::FromStr};

use ec3api::models::Gwp;
use rusqlite::{Connection, Result};

use crate::{settings, Material};
pub fn connection() -> Connection {
    let conn = Connection::open(settings::SettingsProvider::cache_dir().join("carbon.db")).unwrap();
    conn
}
pub fn load_category(category: &str) -> Result<Vec<Material>> {
    let conn = Connection::open(settings::SettingsProvider::cache_dir().join("carbon.db"))?;

    let mut stmt = conn.prepare(
        r"SELECT 
            materials.id, materials.name, materials.description, materials.gwp, materials.gwp_unit, categories.name, categories.display_name, categories.id, categories.description, manufacturers.name, manufacturers.country FROM materials
        JOIN categories ON materials.category_id = categories.id
        LEFT JOIN manufacturers ON materials.manufacturer_name = manufacturers.name
        WHERE categories.name = (?1)
        OR categories.parent_id = (?1);
        "
    )?;

    let mut materials = Vec::new();
    let rows = stmt.query_map([category], f)?;

    for row in rows {
        materials.push(row?);
    }
    Ok(materials)
}

fn f(row: &rusqlite::Row<'_>) -> Result<Material> {
    let name: String = row.get(1)?;
    let id: String = row.get(0)?;
    let description: String = row.get(2)?;
    let gwp: f64 = row.get(3)?;
    let gwp_unit: String = row.get(4)?;
    let category_name: String = row.get(5)?;
    let category_display_name: String = row.get(6)?;
    let category_id: String = row.get(7)?;
    let category_description: String = row.get(8)?;
    let manufacturer_name: String = row.get(9)?;
    let manufacturer_country: String = row.get(10)?;

    Ok(ec3api::models::Ec3Material {
        name,
        gwp: Gwp {
            value: gwp,
            unit: match ec3api::models::GwpUnits::from_str(&gwp_unit) {
                Ok(unit) => unit,
                Err(_) => ec3api::models::GwpUnits::Unknown,
            },
        },
        image: None,
        manufacturer: ec3api::models::Manufacturer {
            name: manufacturer_name,
            country: Some(manufacturer_country),
        },
        description,
        category: ec3api::models::Category {
            description: category_description,
            name: category_name,
            display_name: category_display_name,
            id: category_id,
        },
        id,
    })
}
pub fn migrate() -> Result<()> {
    let conn = Connection::open(settings::SettingsProvider::cache_dir().join("carbon.db"))?;
    conn.execute(
        r"
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            display_name TEXT,
            description TEXT,
            parent_id TEXT
);",
        (),
    )?;
    conn.execute(
        r"CREATE TABLE IF NOT EXISTS manufacturers (
            name TEXT PRIMARY KEY,
            country TEXT
        );",
        (),
    )?;
    conn.execute(
        r"CREATE TABLE if not exists materials (
            id                  TEXT PRIMARY KEY,
            name                TEXT NOT NULL,
            description         TEXT,
            category_id         TEXT NOT NULL,
            gwp                 REAL,
            gwp_unit            TEXT,
            manufacturer_name   TEXT,
            FOREIGN KEY(category_id) 
              REFERENCES categories (id),
            FOREIGN KEY(manufacturer_name)
              REFERENCES manufacturers (name)
        );",
        (), // empty list of parameters.
    )?;

    Ok(())
}
pub fn write(materials: &Vec<Material>, parent: &str) -> Result<()> {
    let conn = Connection::open(settings::SettingsProvider::cache_dir().join("carbon.db"))?;

    // create a set of categories
    let categories: Vec<&ec3api::models::Category> = materials
        .iter()
        .map(|m| &m.category)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut stmt = conn.prepare(
        "INSERT INTO categories (id, name, display_name, description, parent_id) VALUES (?1, ?2, ?3, ?4, ?5);",
    )?;
    for any in categories {
        match stmt.execute([
            &any.id,
            &any.name,
            &any.display_name,
            &any.description,
            parent,
        ]) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "WARNING: Not possible to write category {:?} to db. {e}",
                    &any
                );
            }
        };
    }

    // create set of manufacurers
    let manufacturers: Vec<&ec3api::models::Manufacturer> = materials
        .iter()
        .map(|m| &m.manufacturer)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    // write
    let mut stmt = conn.prepare("INSERT INTO manufacturers (name, country) VALUES (?1, ?2)")?;

    for manu in manufacturers {
        let country = &manu.country.clone().unwrap_or("Unknown".to_string());
        match stmt.execute([&manu.name, &country]) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "WARNING: Not possible to write manufacturer {:?} to db. {}",
                    &manu, e
                );
            }
        };
    }
    println!("Inserting materials");

    let mut stmt = conn.prepare(
            "INSERT INTO materials (id, name, description, category_id, gwp, gwp_unit, manufacturer_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for material in materials {
        let _ = stmt
            .execute([
                &material.id,
                &material.name,
                &material.description,
                &material.category.id,
                &material.gwp.value.to_string(),
                &format!("{:?}", &material.gwp.value),
                &material.manufacturer.name,
            ])
            .map_err(|e| {
                eprintln!(
                    "WARNING: Not possible to write material {:?} into db. {e}",
                    material.name
                )
            });
    }

    Ok(())
}

pub fn query_material_name(input: &str) -> Result<Vec<ec3api::models::Ec3Material>> {
    let conn = Connection::open(settings::SettingsProvider::cache_dir().join("carbon.db"))?;

    let mut query = String::from(input);
    query.insert(0, '%');
    query.push('%');

    let mut stmt = conn.prepare(
        r"SELECT 
            materials.id, materials.name, materials.description, materials.gwp, materials.gwp_unit, categories.name, categories.display_name, categories.id, categories.description, manufacturers.name, manufacturers.country FROM materials
        JOIN categories ON materials.category_id = categories.id
        LEFT JOIN manufacturers ON materials.manufacturer_name = manufacturers.name
        WHERE materials.name LIKE (?1)
        LIMIT 200;
        "
    )?;

    let mut materials = Vec::new();
    let rows = stmt.query_map([&query], f)?;

    for row in rows {
        materials.push(row?);
    }
    Ok(materials)
}
