pub mod jobs;
pub mod material_db;
pub mod project;
pub mod settings;
use std::{
    collections::{BTreeSet, HashSet},
    fmt::Display,
    sync::mpsc::{channel, Receiver},
    thread,
};

use egui_notify::{Anchor, Toasts};

use ec3api::{
    material_filter::MaterialFilter,
    models::{Ec3Category, Ec3Material, Node},
    Ec3Result,
};

extern crate ec3api;

pub type CategoriesTree = ec3api::models::Node<ec3api::models::Ec3Category>;

pub type Material = ec3api::models::Ec3Material;

pub struct State {
    pub materials_loaded: bool,
    pub materials: Vec<Ec3Material>,
    pub categories: Option<Node<Ec3Category>>,
    pub loaded_categories: BTreeSet<String>,
    pub filter_input: String,
    pub fetch_input: String,
    pub country: String,
    pub sort_by: SortBy,
    pub active_tab: Tabs,
    pub selected_category: String,
    materials_rx: Option<Receiver<Vec<Ec3Material>>>,
    categories_rx: Option<Receiver<Node<Ec3Category>>>,
    pub selected: Option<Ec3Material>,
    pub api_key: String,
    pub toasts: Toasts,
    pub category_stats: Option<f64>,
    pub project: Option<project::Project>,
}

impl State {
    pub fn new(api_key: String) -> State {
        State {
            materials_loaded: false,
            materials: Vec::new(),
            loaded_categories: BTreeSet::new(),
            categories: None,
            filter_input: String::new(),
            fetch_input: String::new(),
            api_key,
            sort_by: SortBy::Name,
            active_tab: Tabs::Search, // the initial tab
            selected_category: String::new(),
            materials_rx: None,
            categories_rx: None,
            country: String::new(),
            selected: None,
            toasts: Toasts::default().with_anchor(Anchor::BottomRight),
            category_stats: None,
            project: None,
        }
    }

    /// Loads Categories
    /// # Panics
    /// Panics if .env is missing or incomplete.
    pub fn load_categories(&mut self) {
        let (categories_tx, categories_rx) = channel::<Node<Ec3Category>>();

        self.categories_rx = Some(categories_rx);

        let api_key = self.api_key.to_owned();
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

    /// Search materials by the input field given in [self]
    pub fn fetch_materials_from_input(&mut self) {
        let input = self.fetch_input.clone();
        // deprecated
        // self.fetch_materials(&category);
        // self.load_by_category(&category);
        self.search_by_name(&input);
    }

    /// Spawns thread to fetch materials
    #[deprecated]
    #[allow(dead_code)]
    pub fn fetch_materials(&mut self, category: &str) {
        let mut mf = MaterialFilter::of_category(&category);
        self.materials_loaded = false;
        mf.add_filter("jurisdiction", "in", vec!["150"]);

        let (materials_tx, materials_rx) = channel::<Vec<Ec3Material>>();

        self.materials_rx = Some(materials_rx);

        let api_key = self.api_key.to_owned();
        thread::spawn(move || {
            if let Ok(materials) = ec3api::Ec3api::new(&api_key)
                .endpoint(ec3api::Endpoint::Materials)
                .cache_dir(settings::SettingsProvider::cache_dir())
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
                    let mut seen: HashSet<String> = HashSet::new();
                    let mut filtered = Vec::from(materials);

                    filtered.retain(|x| {
                        let id = x.id.clone();
                        seen.insert(id)
                    });

                    self.loaded_categories = filtered
                        .iter()
                        .map(|mat| mat.category.name.clone())
                        .collect::<BTreeSet<_>>();
                    self.materials = filtered;
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

    /// Tries receiving the categories from the Message receiver
    /// and returns True while a message_receiver exists
    /// and its trying to receive (aka when loading).
    pub fn preload_categories(&mut self) -> bool {
        if let Some(_categories) = &self.categories {
            return false;
        }
        if let Some(rx) = &self.categories_rx {
            match rx.try_recv() {
                Ok(categories) => {
                    println!("Received categories");
                    self.categories = Some(categories);
                    return false;
                }
                Err(_) => {
                    return true;
                }
            }
        }
        false
    }

    pub fn save_materials(&mut self) {
        let _ = material_db::write(&self.materials, &self.fetch_input)
            .map_err(|e| eprintln!("ERROR: {}", e));
    }

    /// Loads a Vec<Material> from the db into state from a given category
    pub fn load_by_category(&mut self, category: &str) {
        let result = material_db::load_category(category);
        match result {
            Ok(_materials) => {
                self.loaded_categories = _materials
                    .iter()
                    .map(|mat| mat.category.name.clone())
                    .collect::<BTreeSet<_>>();
                self.materials = _materials;
                self.materials_loaded = true;
            }
            Err(e) => eprintln!("ERROR: {}", e),
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

    /// Performs a search on the database by the given input, loading internally the returned vector of materials
    fn search_by_name(&mut self, input: &str) -> () {
        if input.is_empty() {
            return ();
        }
        let result = material_db::query_materials(input);
        match result {
            Ok(_materials) => {
                self.loaded_categories = _materials
                    .iter()
                    .map(|mat| mat.category.name.clone())
                    .collect::<BTreeSet<_>>();

                self.materials = _materials;
                self.materials_loaded = true;
            }
            Err(e) => eprintln!("ERROR: {}", e),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SortBy {
    Name,
    Gwp,
}

#[derive(PartialEq, Eq)]
pub enum Tabs {
    Search,
    List,
    Chart,
    Category,
    Calculate,
}

impl Display for Tabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tabs::List => write!(f, "List"),
            Tabs::Chart => write!(f, "Chart"),
            Tabs::Category => write!(f, "Category"),
            Tabs::Search => write!(f, "Search"),
            Tabs::Calculate => write!(f, "Calculate"),
        }
    }
}
