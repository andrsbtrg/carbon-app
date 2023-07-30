use ec3api::Ec3Material;

extern crate ec3api;

pub struct State {
    pub materials_loaded: bool,
    pub materials: Vec<Ec3Material>,
    pub search_input: String,
    pub sort_by: SortBy,
    pub active_tab: Tabs,
    api_key: String,
}

impl State {
    pub fn new(api_key: String) -> State {
        State {
            materials_loaded: false,
            materials: Vec::new(),
            search_input: String::new(),
            api_key,
            sort_by: SortBy::Name,
            active_tab: Tabs::List,
        }
    }

    /// Fetches materials for [`MaterialWindow`].
    ///
    /// # Panics
    ///
    /// Panics if .env is missing or incomplete.
    pub fn load_materials(&mut self) {
        if let Ok(materials) = ec3api::Ec3api::new(&self.api_key)
            // .country(ec3api::Country::Germany)
            .endpoint(ec3api::Endpoint::Materials)
            .fetch()
        {
            self.materials = materials;
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

pub enum Tabs {
    List,
    Chart,
}
