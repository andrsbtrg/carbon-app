use std::{
    sync::mpsc::{channel, Receiver},
    thread,
};

use ec3api::{
    models::{Ec3Category, Node},
    Ec3Result,
};

use crate::{
    material_db::{migrate, write},
    settings,
};

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
        runner.fetch_categories(api_key);
        loop {
            if runner.categories_loaded() {
                break;
            }
        }
        // for each high level category
        if let Some(categories) = runner.categories {
            if let Some(children) = categories.children {
                for cat in &children {
                    let query = cat.value.name.clone();
                    let mut mf = ec3api::material_filter::MaterialFilter::of_category(&query);
                    mf.add_filter("jurisdiction", "in", vec!["150"]);

                    let materials = ec3api::Ec3api::new(api_key)
                        .endpoint(ec3api::Endpoint::Materials)
                        .material_filter(mf)
                        .cache_dir(settings::SettingsProvider::cache_dir())
                        .fetch()
                        .map_err(|e| {
                            eprintln!("ERROR: {:?}", e);
                            CError::FromApi
                        })?;
                    println!("Finished fetching {}", &query);
                    write(&materials, &query).map_err(|e| {
                        eprintln!("ERROR: while writing to db: {}", e);
                        CError::FromDb
                    })?;
                }
            } else {
                eprintln!("Failed to fetch.")
            }
        }

        println!("Done updating DB!");
        Ok(())
    }
}
