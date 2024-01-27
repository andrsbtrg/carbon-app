use std::{collections::HashSet, str::FromStr};

use ec3api::models::{DeclaredUnit, Ec3Category, Gwp};
use rusqlite::{Connection, Result};

use crate::{settings, Material};
pub fn connection() -> Result<Connection> {
    Connection::open(settings::SettingsProvider::default_path().join("carbon.db"))
}
pub fn write_category(category: &Ec3Category) -> Result<()> {
    let conn = connection()?;

    let mut stmt = conn
        .prepare("UPDATE categories SET declared_value=(?1), declared_unit=(?2) WHERE id=(?3);")?;

    match stmt.execute([
        &category.declared_unit.value.to_string(),
        &format!("{:?}", &category.declared_unit.unit),
        &category.id,
    ]) {
        Ok(_) => {
            println!("Update category {} into db.", { &category.name });
            Ok(())
        }
        Err(e) => {
            eprintln!("ERROR: Updating category {}", &category.name);
            Err(e)
        }
    }
}
pub fn load_category(category: &str) -> Result<Vec<Material>> {
    let conn = connection()?;
    let mut stmt = conn.prepare(
        r"SELECT 
            materials.id, materials.name, materials.description, materials.gwp, materials.gwp_unit, categories.name, categories.display_name, categories.id, categories.description, manufacturers.name, manufacturers.country, materials.declared_value, materials.declared_unit FROM materials
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

fn g(row: &rusqlite::Row<'_>) -> Result<DeclaredUnit> {
    let declared_value: f64 = row.get(0)?;
    let declared_unit: String = row.get(1)?;
    Ok(DeclaredUnit {
        value: declared_value,
        unit: ec3api::models::Unit::from_str(&declared_unit)
            .unwrap_or(ec3api::models::Unit::Unknown),
    })
}
fn f(row: &rusqlite::Row<'_>) -> Result<Material> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let description: String = row.get(2)?;
    let gwp: f64 = row.get(3)?;
    let gwp_unit: String = row.get(4)?;
    let category_name: String = row.get(5)?;
    let category_display_name: String = row.get(6)?;
    let category_id: String = row.get(7)?;
    let category_description: String = row.get(8)?;
    let manufacturer_name: String = row.get(9)?;
    let manufacturer_country: String = row.get(10)?;
    let declared_value: f64 = row.get(11)?;
    let declared_unit: String = row.get(12)?;

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
        declared_unit: ec3api::models::DeclaredUnit {
            value: declared_value,
            unit: ec3api::models::Unit::from_str(&declared_unit)
                .unwrap_or(ec3api::models::Unit::Unknown),
        },
    })
}
pub fn migrate() -> Result<()> {
    let conn = connection()?;
    conn.execute(
        r"
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            display_name TEXT,
            description TEXT,
            parent_id TEXT,
            declared_value REAL,
            declared_unit TEXT    
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
            declared_value      REAL,
            declared_unit       TEXT,           
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
    let conn = connection()?;

    // create a set of categories
    let categories: Vec<&ec3api::models::Category> = materials
        .iter()
        .map(|m| &m.category)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut stmt = conn.prepare("INSERT INTO categories (id, name, display_name, description, parent_id) VALUES (?1, ?2, ?3, ?4, ?5);",
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
            "INSERT INTO materials (id, name, description, category_id, gwp, gwp_unit, manufacturer_name, declared_value, declared_unit) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    for material in materials {
        let _ = stmt
            .execute([
                &material.id,
                &material.name,
                &material.description,
                &material.category.id,
                &material.gwp.value.to_string(),
                &format!("{:?}", &material.gwp.unit),
                &material.manufacturer.name,
                &material.declared_unit.value.to_string(),
                &format!("{:?}", &material.declared_unit.unit),
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

/// Searches database for materials by name, category or parent category
pub fn query_materials(input: &str) -> Result<Vec<ec3api::models::Ec3Material>> {
    let conn = connection()?;
    let mut query = String::from(input);
    query.insert(0, '%');
    query.push('%');

    let mut stmt = conn.prepare(
        r"SELECT 
            materials.id, materials.name, materials.description, materials.gwp, materials.gwp_unit, categories.name, categories.display_name, categories.id, categories.description, manufacturers.name, manufacturers.country, materials.declared_value, materials.declared_unit FROM materials
        JOIN categories ON materials.category_id = categories.id
        LEFT JOIN manufacturers ON materials.manufacturer_name = manufacturers.name
        WHERE materials.name LIKE (?1)
        OR categories.parent_id LIKE (?1)
        OR categories.name LIKE (?1)
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

pub fn get_category_stats(category: &ec3api::models::Category) -> Result<f64> {
    let conn = connection()?;
    let mut stmt = conn.prepare(
        "
SELECT avg(gwp) from materials
WHERE category_id = (?1);
",
    )?;
    let mut response: f64 = 0.;
    if let Some(row) = stmt.query_map([&category.id], |row| row.get(0))?.next() {
        response = row?;
    }
    Ok(response)
}

pub fn get_category_avg(category: &str) -> Result<f64> {
    let conn = connection()?;
    let mut stmt = conn.prepare(
        "
SELECT avg(gwp) from materials
JOIN categories on categories.id = materials.category_id
WHERE categories.name = (?1);
",
    )?;
    let mut response = 0.;
    if let Some(row) = stmt.query_map([category], |row| row.get(0))?.next() {
        response = row?;
    }
    Ok(response)
}

pub fn get_category_unit(category: &str) -> Result<DeclaredUnit> {
    let conn = connection()?;
    let mut stmt = conn.prepare(
        "
SELECT declared_value, declared_unit FROM categories
WHERE categories.name = (?1);
",
    )?;
    let row = stmt.query_map([category], g)?.next().unwrap();
    let unit: DeclaredUnit = row?;
    Ok(unit)
}
