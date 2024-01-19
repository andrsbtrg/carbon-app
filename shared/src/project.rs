use ec3api::models::Ec3Material;

pub struct Project {
    pub components: Vec<Component>,
}
impl Project {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
}

pub struct Component {
    pub quantity: f64,
    pub calculated: f64,
    pub material: Ec3Material,
    pub category_avg: f64,
}
impl Component {
    pub fn calculate(&mut self) -> () {
        self.calculated = self.quantity * self.material.gwp.value;
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
