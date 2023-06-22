use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::{fmt::Debug, str::FromStr};
use thiserror::Error;

const BASE_PATH: &str = "https://buildingtransparency.org/api/";

pub enum Country {
    Us,
    Germany,
    UK,
    None,
}
impl ToString for Country {
    fn to_string(&self) -> String {
        match &self {
            Country::Us => "US".to_string(),
            Country::Germany => "DE".to_string(),
            Country::UK => "UK".to_string(),
            Country::None => "".to_string(),
        }
    }
}
pub enum Endpoint {
    Materials,
}
impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match &self {
            Endpoint::Materials => "materials".to_string(),
        }
    }
}

pub struct Ec3api {
    api_key: String,
    endpoint: Endpoint,
    country: Country,
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
    value: f64,
    unit: GwpUnits,
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
        // dbg!(x,y);
        let value = x.parse::<f64>().map_err(|_| ApiError::GwpError)?;
        let unit = y.parse::<GwpUnits>().map_err(|_| ApiError::GwpError)?;
        // dbg!(&value, &unit);
        Ok(Gwp { value, unit })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manufacturer {
    pub name: String,
    pub country: String,
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

fn write_cache(json: String) {
    match std::fs::write("cache.json", &json) {
        Ok(_) => {
            println!("Results cached")
        }
        Err(e) => {
            println!("Could not write JSON file: {e:?}")
        }
    };
}
fn read_cache() -> Result<Vec<Ec3Material>, ApiError> {
    let contents = std::fs::read_to_string("cache.json")?;

    let result: Value = serde_json::from_str(&contents).unwrap();

    let mut out: Vec<Ec3Material> = Vec::new();

    result
        .as_array()
        .ok_or(ApiError::EmptyArray())?
        .iter()
        .for_each(|m| {
            // dbg!(m);
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
                    country: m["manufacturer"]["country"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
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
        }
    }

    pub fn country(&mut self, country_code: Country) -> &mut Self {
        self.country = country_code;

        self
    }

    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoint = endpoint;

        self
    }
    fn prepare_url(&self) -> Result<String, ApiError> {
        let jurisdiction = match self.country {
            Country::None => "".to_owned(),
            _ => format!("?jurisdiction={}", self.country.to_string()),
        };
        let url = format!("{}{}{}", BASE_PATH, self.endpoint.to_string(), jurisdiction);

        Ok(url)
    }
    pub fn fetch(&mut self) -> Result<Vec<Ec3Material>, ApiError> {
        if let Ok(ret) = read_cache() {
            return Ok(ret);
        } else {
            println!("no cache found");
        }

        println!("Querying materials...");

        let path = self.prepare_url().unwrap();

        let auth = format!("{} {}", "Bearer", self.api_key);

        let response = ureq::get(&path)
            .set("Authorization", &auth)
            // .set("X-Total-Count", "1")
            .call()?
            .into_string()?;

        let json: Value = serde_json::from_str(&response)?;

        let mut materials: Vec<Ec3Material> = Vec::new();

        // let val = &json.as_array().expect("not an array")[0];
        // println!("{:?}", val);
        // let mat: Ec3Material = serde_json::from_value(val.to_owned()).unwrap();
        // dbg!(mat);

        json.as_array()
            .ok_or(ApiError::EmptyArray())?
            .iter()
            .for_each(|v| {
                // let gwp = Gwp::from_str(v["gwp"].as_str().unwrap()).unwrap_or_default();
                // let name: String = v.get("name").unwrap().to_string().replace("\"", "");
                // let image = v.get("image").unwrap_or(&json!("".to_string())).as_str().unwrap_or("<No Image>").to_string();
                // let country = v.get("country").unwrap_or.to_string();
                // println!("{}", country);
                dbg!(v);
                let material: Ec3Material = serde_json::from_value(v.to_owned()).unwrap();

                materials.push(material);
            });
        match serde_json::to_string_pretty(&materials) {
            Ok(json) => write_cache(json),
            Err(e) => {
                eprint!("Error: could not write cache: {e:?}");
            }
        };

        // dbg!(materials);
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
