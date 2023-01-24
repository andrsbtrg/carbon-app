use std::fmt::Debug;
use serde_json::Value;

const BASE_PATH: &str = "https://buildingtransparency.org/api/";

pub struct Ec3api {
    api_key: String,
}

#[derive(Debug)]
pub struct Ec3Material {
    pub name: String,
    pub gwp: String,
    pub image: String,
}

impl Ec3api {
    pub fn new(api_key: &str) -> Self {
        Ec3api {
            api_key: api_key.to_string(),
        }
    }

    pub fn get_epds(&self) -> Option<Vec<Ec3Material>> {
        let path = BASE_PATH.to_string() + "epds";
        let auth = format!("{} {}", "Bearer", self.api_key);


        let response = ureq::get(&path)
            .set("Authorization", &auth)
            // .set("X-Total-Count", "1")
            .call()
            .unwrap();

        let json: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();

        let mut materials: Vec<Ec3Material> = Vec::new();

        json.as_array()
            .unwrap()
            .iter()
            .for_each(|v| {

                let material = Ec3Material {
                    name: v["name"].to_string(),
                    gwp: v["gwp"].to_string(),
                    image: v["image"].to_string(),
                };
                materials.push(material);

                // dbg!(&v["name"]);

            });

        // dbg!(materials);
        Some(materials)

    }
}

impl Debug for Ec3api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ec3api").field("api_key", &self.api_key).finish()
    }
}
