use ec3api::models::{DeclaredUnit, Ec3Material, Gwp, GwpUnits};

use crate::material_db;

pub struct Project {
    pub components: Vec<Component>,
    pub calculated_gwp: f64,
}
impl Project {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            calculated_gwp: 0.,
        }
    }
    pub fn calculate(&mut self) -> () {
        self.calculated_gwp = self.components.iter().map(|c| c.calculated).sum::<f64>()
    }
}

pub trait Material {
    fn get_unit(&self) -> &DeclaredUnit;
    fn get_gwp(&self) -> &Gwp;
    fn get_name(&self) -> &str;
}

impl Material for Ec3Material {
    fn get_unit(&self) -> &DeclaredUnit {
        &self.declared_unit
    }

    fn get_gwp(&self) -> &Gwp {
        &self.gwp
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
pub struct UMaterial {
    pub name: String,
    pub gwp: Gwp,
    pub unit: DeclaredUnit,
}

impl UMaterial {
    pub fn get_from_db(category: &str) -> Self {
        let cat_avg = material_db::get_category_by_name(category).unwrap_or(0.);
        Self {
            name: category.to_string(),
            gwp: Gwp {
                value: cat_avg,
                unit: GwpUnits::KgCO2e,
            },
            unit: DeclaredUnit {
                value: 1.,
                unit: ec3api::models::Unit::Unknown,
            },
        }
    }
}
impl Material for UMaterial {
    fn get_unit(&self) -> &DeclaredUnit {
        &self.unit
    }

    fn get_gwp(&self) -> &Gwp {
        &self.gwp
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

pub struct Component {
    pub quantity: f64,
    pub calculated: f64,
    pub material: Box<dyn Material>,
    pub category_avg: f64,
}
pub enum CmpResult {
    AlmostEqual,
    Greater,
    Smaller,
}
impl Component {
    pub fn calculate(&mut self) -> () {
        // to normalize: since the GWP value inside of material is per declared_unit
        // If the declared_unit is 1.5 Kg means the calculated value is qt * gwp / 1.5 kg
        self.calculated =
            self.quantity * self.material.get_gwp().value / self.material.get_unit().value;
    }

    pub fn cmp_to_average(&self) -> CmpResult {
        if self.material.get_gwp().value > 1.25 * self.category_avg {
            CmpResult::Greater
        } else if self.material.get_gwp().value < 0.75 * self.category_avg {
            CmpResult::Smaller
        } else {
            CmpResult::AlmostEqual
        }
    }
}

impl Project {
    pub fn add_component(&mut self, selected: Ec3Material, category_avg: f64) {
        self.components.push(Component {
            quantity: 0.,
            calculated: 0.,
            material: Box::new(selected),
            category_avg,
        });
    }

    pub fn add_generic_comp(&mut self, cat: &str) -> () {
        let material = UMaterial::get_from_db(cat);
        let category_avg = material.gwp.value;
        self.components.push(Component {
            quantity: 0.,
            calculated: 0.,
            material: Box::new(material),
            category_avg,
        });
    }
}
