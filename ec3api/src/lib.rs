use std::{fmt::Debug, error, str::FromStr};
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use thiserror::Error;

const BASE_PATH: &str = "https://buildingtransparency.org/api/";

pub struct Ec3api {
    api_key: String,
}
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Request failed")]
    RequestError(#[from] ureq::Error),

    #[error("Could not read cache")]
    CacheError(#[from] std::io::Error),

    #[error("Could not deserialize")]
    Deserialize(#[from] serde_json::Error),

    #[error("Request succeded but the material list is empty")]
    EmptyArray(),

    #[error("Wrong GWP format")]
    GwpError,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GwpUnits {
    KgCO2e,
    Unknown 
}

impl FromStr for GwpUnits {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "KgCO2e" => Ok(Self::KgCO2e),
                "kgCO2e" => Ok(Self::KgCO2e),
                "kgCo2e" => Ok(Self::KgCO2e),
                _ => Ok(Self::Unknown)
            }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gwp {
    value: f64,
    unit: GwpUnits
}

impl Gwp {
    pub fn as_str(&self) -> String {
        format!("{} {:?}", self.value, self.unit)
    }
}

impl Into<String> for Gwp {
    fn into(self) -> String {
        format!("{} {:?}", self.value, self.unit)
    }
}

impl Default for Gwp {
    fn default() -> Self {
        Self { value:0 as f64, unit: GwpUnits::KgCO2e }
    }
}

impl FromStr for Gwp {
        type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .split_once(' ')
            .ok_or(ApiError::GwpError)?;
        // dbg!(x,y);
        let value = x.parse::<f64>().map_err(|_| ApiError::GwpError)?;
        let unit = y.parse::<GwpUnits>().map_err(|_| ApiError::GwpError)?;
        dbg!(&value, &unit);
        Ok(Gwp{ value, unit })
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Ec3Material {
    pub name: String,
    pub gwp: Gwp,
    pub image: String,
}

impl Ec3Material {
    fn from_json(value: &serde_json::Value) -> Self {
        
        todo!()

    }
}

fn write_cache(json: String) {

    match std::fs::write("cache.json", &json) {
        Ok(_) => {println!("Results cached")},
        Err(e) => {println!("{e:?}")},
    };
}
fn read_cache() -> Result<Vec<Ec3Material>, ApiError> {

    let contents = std::fs::read_to_string("cache.json")?;
    
    let result: Vec<Ec3Material> = serde_json::from_str(&contents)?;
    
    Ok(result)

}

impl Ec3api {
    pub fn new(api_key: &str) -> Self {
        Ec3api {
            api_key: api_key.to_string(),
        }
    }

    pub fn get_epds(&self) -> Result<Vec<Ec3Material>, ApiError> {

        if let Ok(ret) = read_cache() {
            return Ok(ret)
        }

        println!("Querying materials...");

        let path = BASE_PATH.to_string() + "materials";

        let auth = format!("{} {}", "Bearer", self.api_key);


        let response = ureq::get(&path)
            .set("Authorization", &auth)
            // .set("X-Total-Count", "1")
            .call()?
            .into_string()?;
        
        let json: Value = serde_json::from_str(&response)?;

        let mut materials: Vec<Ec3Material> = Vec::new();

        json.as_array()
            .ok_or(ApiError::EmptyArray())?
            .iter()
            .for_each(|v| {
                
                let material = Ec3Material {
                    name: v.get("name").unwrap().as_str().unwrap().to_string(),
                    gwp: Gwp::from_str(v["gwp"].as_str().unwrap()).unwrap_or_default(),
                    image: v.get("image").unwrap_or(&json!("".to_string())).as_str().unwrap_or("<No Image>").to_string(),
                };

                materials.push(material);

            });
        match serde_json::to_string_pretty(&materials) {
            Ok(json) => {write_cache(json)},
            Err(e) => {dbg!(e);},
        };

        // dbg!(materials);
        Ok(materials)

    }
}

impl Debug for Ec3api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ec3api").field("api_key", &self.api_key).finish()
    }
}
