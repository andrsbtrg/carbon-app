use std::{
    fmt::Display,
    sync::mpsc::{channel, Receiver},
    thread,
};

use ec3api::material_filter::MaterialFilter;
use ec3api::models::Ec3Material;

extern crate ec3api;

pub struct State {
    pub materials_loaded: bool,
    pub materials: Vec<Ec3Material>,
    pub search_input: String,
    pub fetch_input: String,
    pub country: String,
    pub sort_by: SortBy,
    pub active_tab: Tabs,
    pub selected_category: String,
    pub materials_rx: Option<Receiver<Vec<Ec3Material>>>,
    api_key: String,
}

impl State {
    pub fn new(api_key: String) -> State {
        State {
            materials_loaded: false,
            materials: Vec::new(),
            search_input: String::new(),
            fetch_input: String::new(),
            api_key,
            sort_by: SortBy::Name,
            active_tab: Tabs::List,
            selected_category: String::new(),
            materials_rx: None,
            country: String::new(),
        }
    }

    /// Fetches materials for [`MaterialWindow`].
    /// # Panics
    /// Panics if .env is missing or incomplete.
    pub fn load_materials(&mut self) {
        let (materials_tx, materials_rx) = channel::<Vec<Ec3Material>>();

        let mut mf = MaterialFilter::of_category("Wood");
        self.materials_rx = Some(materials_rx);
        mf.add_filter("jurisdiction", "in", vec!["150"]);

        let api_key = self.api_key.to_owned();
        thread::spawn(move || {
            if let Ok(materials) = ec3api::Ec3api::new(&api_key)
                .material_filter(mf)
                .endpoint(ec3api::Endpoint::Materials)
                .fetch()
            {
                // Send materials to the receiver
                println!("Finished fetching materials.");
                if let Err(e) = materials_tx.send(materials) {
                    println!("ERROR: {:?}", e);
                }
            }
        });
    }

    pub fn search_materials(&mut self) {
        let mut mf = MaterialFilter::of_category(&self.fetch_input);
        self.materials_loaded = false;
        mf.add_filter("jurisdiction", "in", vec!["150"]);

        let (materials_tx, materials_rx) = channel::<Vec<Ec3Material>>();

        self.materials_rx = Some(materials_rx);

        let api_key = self.api_key.to_owned();
        thread::spawn(move || {
            if let Ok(materials) = ec3api::Ec3api::new(&api_key)
                .endpoint(ec3api::Endpoint::Materials)
                .material_filter(mf)
                .fetch()
            {
                // Send materials to the receiver
                println!("Finished fetching materials.");
                if let Err(e) = materials_tx.send(materials) {
                    println!("ERROR: {:?}", e);
                }
            }
        });
    }

    /// Tries receiving the materials from the Message receiver
    /// and returns True while a message_receiver exists
    /// and its trying to receive (aka when loading).
    pub fn preload_data(&mut self) -> bool {
        if self.materials_loaded {
            return false;
        }
        if let Some(rx) = &self.materials_rx {
            match rx.try_recv() {
                Ok(materials) => {
                    println!("Received materials");
                    self.materials = materials;
                    self.materials_loaded = true;
                    return false;
                }
                Err(_) => {
                    return true;
                }
            }
        } else {
            false
        }
    }

    pub fn sort_by(&mut self, op: SortBy) {
        match op {
            SortBy::Gwp => self
                .materials
                .sort_by(|a, b| a.gwp.value.total_cmp(&b.gwp.value)),
            SortBy::Name => self.materials.sort_by(|a, b| a.name.cmp(&b.name)),
        };
        self.sort_by = op;
    }
}

#[derive(PartialEq, Eq)]
pub enum SortBy {
    Name,
    Gwp,
}

#[derive(PartialEq, Eq)]
pub enum Tabs {
    List,
    Chart,
}

impl Display for Tabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tabs::List => write!(f, "List"),
            Tabs::Chart => write!(f, "Chart"),
        }
    }
}
