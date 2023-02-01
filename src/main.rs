use ec3api::{self, Ec3Material};
use eframe::{run_native, NativeOptions, epaint::Vec2, egui::{CentralPanel, ScrollArea}};

use std::env;

struct MaterialWindow {
    materials: Vec<MaterialsData>
}

impl MaterialWindow {
    fn new(materials: &Vec<Ec3Material>) -> Self {

        let collection: Vec<MaterialsData> = materials.iter()
            .map(|m| {
            MaterialsData {
                title: m.name.to_string(),
                descr: m.gwp.as_str(),
                country: m.manufacturer.country.to_owned(),
                // img_url: m.image.to_owned().unwrap_or("<No image>".to_string()),
            }
        }).collect();

        MaterialWindow { materials: collection }

    }
        
    fn render_material_cards(&self, ui: &mut eframe::egui::Ui ) {
        for m in &self.materials {
            ui.add_space(2.);
            
            ui.label(&m.title);
            ui.monospace(&m.descr);

            // ui.hyperlink(&m.img_url); // removed for now
            ui.monospace(&m.country);
            ui.add_space(2.);
            ui.separator();
            
        }
    }
}

struct MaterialsData {
    title: String,
    descr: String,
    // img_url: String,
    country: String,
}

impl eframe::App for MaterialWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {

            ScrollArea::vertical().auto_shrink([false;2]).show(ui, |ui| {
                
                self.render_material_cards(ui);
                
            });
        });
    }

}

fn main() {
    dotenv::dotenv().expect("No .env file found!");
    let api_key = env::var("API_KEY").expect("API Key missing!");


    let materials = ec3api::Ec3api::new(&api_key)
        .set_country("US")
        .set_endpoint("materials")
        .call()
        .unwrap();
    // dbg!(app);

    // let materials = &api.call().unwrap();

    let app = MaterialWindow::new(&materials);
    
    let mut win_options = NativeOptions::default();
    win_options.initial_window_size = Some(Vec2::new(540., 960.));
    win_options.resizable = false;
    win_options.follow_system_theme = false;
    
    // win_options.resizable = false;

    run_native("Materials", win_options, Box::new(|_| Box::new(app)));
}
