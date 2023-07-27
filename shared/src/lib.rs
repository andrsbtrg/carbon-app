extern crate ec3api;

pub struct State {
    pub materials_loaded: bool,
    pub materials: Vec<MaterialsData>,
    pub search_input: String,
    api_key: String,
}

impl State {
    pub fn new(api_key: String) -> State {
        State {
            materials_loaded: false,
            materials: Vec::new(),
            search_input: String::new(),
            api_key,
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
            let collection: Vec<MaterialsData> = materials
                .iter()
                .map(|m| {
                    MaterialsData {
                        title: m.name.as_str().to_owned(),
                        gwp: m.gwp.as_str(),
                        country: m.manufacturer.country.as_str().to_owned(),
                        category: m.category.description.as_str().to_owned(),
                        // img_url: m.image.to_owned().unwrap_or("<No image>".to_string()),
                    }
                })
                .collect();

            self.materials = collection;
        }
    }
}

#[derive(Clone)]
pub struct MaterialsData {
    pub title: String,
    pub gwp: String,
    // img_url: String,
    pub country: String,
    pub category: String,
}
