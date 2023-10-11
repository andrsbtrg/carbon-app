use material_filter::MaterialFilter;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::{
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
};
use thiserror::Error;

use crate::material_filter::convert;

const BASE_PATH: &str = "https://buildingtransparency.org/api/";

pub mod material_filter;

pub enum Country {
    US,
    Germany,
    UK,
    None,
}
impl Display for Country {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            Country::US => write!(f, "US"),
            Country::Germany => write!(f, "DE"),
            Country::UK => write!(f, "UK"),
            Country::None => write!(f, ""),
        }
    }
}
pub enum Endpoint {
    Materials,
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Endpoint::Materials => write!(f, "materials"),
        }
    }
}

pub struct Ec3api {
    api_key: String,
    endpoint: Endpoint,
    country: Country,
    mf: Option<MaterialFilter>,
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
    Unknown,
}

impl FromStr for GwpUnits {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "KgCO2e" => Ok(Self::KgCO2e),
            "kgCO2e" => Ok(Self::KgCO2e),
            "kgCo2e" => Ok(Self::KgCO2e),
            _ => Ok(Self::Unknown),
        }
    }
}
// You can use this deserializer for any type that implements FromStr
// and the FromStr::Err implements Display
fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,                // Required for S::from_str...
    S::Err: std::fmt::Display, // Required for .map_err(de::Error::custom)
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gwp {
    pub value: f64,
    pub unit: GwpUnits,
}

impl Gwp {
    pub fn as_str(&self) -> String {
        format!("{} {:?}", self.value, self.unit)
    }
}

impl Default for Gwp {
    fn default() -> Self {
        Self {
            value: 0 as f64,
            unit: GwpUnits::KgCO2e,
        }
    }
}

impl FromStr for Gwp {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(' ').ok_or(ApiError::GwpError)?;
        let value = x.parse::<f64>().map_err(|_| ApiError::GwpError)?;
        let unit = y.parse::<GwpUnits>().map_err(|_| ApiError::GwpError)?;
        Ok(Gwp { value, unit })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manufacturer {
    pub name: String,
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ec3Material {
    pub name: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub gwp: Gwp,
    #[serde(default)]
    pub image: Option<String>,
    pub manufacturer: Manufacturer,
    pub description: String,
    pub category: Category,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub description: String,
}

fn write_cache(json: String, filename: &str) {
    match std::fs::write(format!("{}.json", filename), &json) {
        Ok(_) => {
            println!("Results cached")
        }
        Err(e) => {
            println!("Could not write JSON file: {e:?}")
        }
    };
}
fn read_cache(category: &str) -> Result<Vec<Ec3Material>, ApiError> {
    let contents = std::fs::read_to_string(format!("{}.json", category))?;

    let result: Value = serde_json::from_str(&contents).unwrap();

    let mut out: Vec<Ec3Material> = Vec::new();

    result
        .as_array()
        .ok_or(ApiError::EmptyArray())?
        .iter()
        .for_each(|m| {
            let material = Ec3Material {
                name: m["name"].as_str().unwrap_or_default().to_string(),
                gwp: Gwp {
                    value: m["gwp"]["value"].as_f64().unwrap_or_default(),
                    unit: GwpUnits::from_str(m["gwp"]["unit"].as_str().unwrap_or_default())
                        .unwrap_or(GwpUnits::Unknown),
                },
                image: Some(m["image"].to_string()),
                manufacturer: Manufacturer {
                    name: m["manufacturer"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    country: Some(
                        m["manufacturer"]["country"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    ),
                },
                description: m["description"].as_str().unwrap_or("").to_string(),
                category: Category {
                    description: m["category"]["description"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                },
            };

            out.push(material);
        });

    Ok(out)
}

impl Ec3api {
    pub fn new(api_key: &str) -> Ec3api {
        Ec3api {
            api_key: api_key.to_string(),
            endpoint: Endpoint::Materials,
            country: Country::Germany,
            mf: None,
        }
    }

    pub fn country(&mut self, country_code: Country) -> &mut Self {
        self.country = country_code;

        self
    }

    pub fn material_filter(&mut self, mf: MaterialFilter) -> &mut Self {
        self.mf = Some(mf);
        self
    }

    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoint = endpoint;

        self
    }
    fn prepare_url(&self) -> String {
        let jurisdiction = match self.country {
            Country::None => "".to_owned(),
            _ => format!("?jurisdiction={}", self.country.to_string()),
        };
        let url = format!("{}{}{}", BASE_PATH, self.endpoint.to_string(), jurisdiction);

        url
    }
    pub fn fetch(&mut self) -> Result<Vec<Ec3Material>, ApiError> {
        let category = match &self.mf {
            Some(mf) => mf.get_category(),
            None => "cache".to_string(),
        };

        if let Ok(ret) = read_cache(&category) {
            return Ok(ret);
        } else {
            println!("no cache found");
        }

        println!("Querying materials...");

        let path = self.prepare_url();

        let auth = format!("Bearer {}", self.api_key);

        let filter = if let Some(mf) = &self.mf {
            convert(mf)
        } else {
            String::new()
        };

        let response = ureq::get(&path)
            .set("Authorization", &auth)
            .query("mf", &filter)
            .call()?
            .into_string()?;

        let json: Value = serde_json::from_str(&response)?;

        let mut materials: Vec<Ec3Material> = Vec::new();

        json.as_array()
            .ok_or(ApiError::EmptyArray())?
            .iter()
            .for_each(|v| {
                let material: Ec3Material = serde_json::from_value(v.to_owned()).unwrap();

                materials.push(material);
            });
        let category = match &self.mf {
            Some(mf) => mf.get_category(),
            None => "cache".to_string(),
        };
        match serde_json::to_string_pretty(&materials) {
            Ok(json) => write_cache(json, &category),
            Err(e) => {
                eprint!("Error: could not write cache: {e:?}");
            }
        };
        Ok(materials)
    }
}

impl Debug for Ec3api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ec3api")
            .field("api_key", &self.api_key)
            .finish()
    }
}
