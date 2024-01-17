use std::{
    sync::mpsc::{channel, Receiver},
    thread,
};

use ec3api::{
    models::{Ec3Category, Ec3Material, Node},
    Ec3Result,
};

use crate::{
    material_db::{migrate, write},
    settings,
};

#[derive(Debug)]
pub enum CError {
    FromApi,
    FromDb,
}
pub type Result<T> = std::result::Result<T, CError>;

pub struct Runner {
    categories_rx: Option<Receiver<Node<Ec3Category>>>,
    pub categories: Option<Node<Ec3Category>>,
}

impl Runner {
    fn categories_loaded(&mut self) -> bool {
        if let Some(_categories) = &self.categories {
            return true;
        }
        if let Some(rx) = &self.categories_rx {
            match rx.try_recv() {
                Ok(categories) => {
                    println!("Received categories");
                    self.categories = Some(categories);
                    return true;
                }
                Err(_) => {
                    return false;
                }
            }
        }
        false
    }
    fn fetch_categories(&mut self, api_key: &str) {
        self.categories = None;
        println!("Starting to fetch categories...");
        let (categories_tx, categories_rx) = channel::<Node<Ec3Category>>();

        let api_key = api_key.to_string();
        self.categories_rx = Some(categories_rx);

        thread::spawn(move || {
            if let Ok(result) = ec3api::Ec3api::new(&api_key)
                .endpoint(ec3api::Endpoint::Categories)
                .fetch_all()
            {
                // Send materials to the receiver
                println!("Finished fetching categories.");
                if let Ec3Result::Categories(categories) = result {
                    if let Err(e) = categories_tx.send(categories) {
                        println!("ERROR: {:?}", e);
                    }
                }
            }
        });
    }
    pub fn update_db(api_key: &str) -> Result<()> {
        let api_key = api_key.to_string();

        thread::spawn(move || -> Result<()> {
            migrate().map_err(|e| {
                eprintln!("ERROR: Not possible to migrate - {:?}", e);
                CError::FromDb
            })?;
            println!("Updating DB...");
            // load categories
            let mut runner = Runner {
                categories_rx: None,
                categories: None,
            };
            runner.fetch_categories(&api_key);
            loop {
                if runner.categories_loaded() {
                    break;
                }
            }
            let category_tree = runner.categories.ok_or_else(|| {
                eprintln!("ERROR: received no category tree.");
                CError::FromApi
            })?;

            let categories = category_tree.children.ok_or_else(|| {
                eprintln!("ERROR: category tree contains no children");
                CError::FromApi
            })?;

            // for each high level category
            traverse_fetch(&categories, &api_key);
            // first_level_fetch(&categories, &api_key)?;
            println!("Done updating DB!");
            Ok(())
        });
        Ok(())
    }
}

#[allow(dead_code)]
fn first_level_fetch(categories: &[Node<Ec3Category>], api_key: &str) -> Result<()> {
    for cat in categories {
        let category = cat.value.name.clone();
        let materials: Vec<Ec3Material> = fetch_category(&api_key, &category)?;
        println!(
            "Received {count} materials from {category}",
            count = materials.len()
        );
        write(&materials, &category).map_err(|e| {
            eprintln!("ERROR: while writing to db: {}", e);
            CError::FromDb
        })?;
    }
    Ok(())
}

fn traverse_fetch(categories: &[Node<Ec3Category>], api_key: &str) -> () {
    for cat in categories {
        let category = cat.value.name.clone();
        let materials: Vec<Ec3Material> = match fetch_category(&api_key, &category) {
            Ok(mat) => mat,
            Err(_) => return (),
        };
        println!(
            "Received {count} materials from {category}",
            count = materials.len()
        );
        let _ = write(&materials, &category).map_err(|e| {
            eprintln!("ERROR: while writing to db: {}", e);
            CError::FromDb
        });
        if cat.children.as_ref().is_some_and(|c| c.len() > 0) {
            let children = &cat.children.as_ref().unwrap();
            traverse_fetch(&children, api_key);
        }
    }
}

fn fetch_category(api_key: &str, query: &str) -> Result<Vec<Ec3Material>> {
    let mut mf = ec3api::material_filter::MaterialFilter::of_category(&query);
    mf.add_filter("jurisdiction", "in", vec!["150"]);

    ec3api::Ec3api::new(&api_key)
        .endpoint(ec3api::Endpoint::Materials)
        .material_filter(mf)
        .cache_dir(settings::SettingsProvider::cache_dir())
        .fetch()
        .map_err(|e| {
            eprintln!(
                "ERROR: while fetching materials from category {query} {:?}",
                e
            );
            CError::FromApi
        })
}
