use ec3api::models::Ec3Material;

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

pub struct Component {
    pub quantity: f64,
    pub calculated: f64,
    pub material: Ec3Material,
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
            self.quantity * self.material.gwp.value / self.material.declared_unit.value;
    }

    pub fn cmp_to_average(&self) -> CmpResult {
        if self.material.gwp.value > 1.25 * self.category_avg {
            CmpResult::Greater
        } else if self.material.gwp.value < 0.75 * self.category_avg {
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
            material: selected,
            category_avg,
        });
    }
}
