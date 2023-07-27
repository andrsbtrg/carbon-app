pub(crate) use ec3api;
use eframe::{
    egui::{self, CentralPanel, ScrollArea, TopBottomPanel},
    epaint::Vec2,
};

use std::env;

struct MaterialWindow {
    materials_loaded: bool,
    materials: Vec<MaterialsData>,
    search_input: String,
}

impl MaterialWindow {
    /// Creates a new [`MaterialWindow`].
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            materials_loaded: false,
            materials: Vec::new(),
            search_input: String::new(),
        }
    }

    /// Fetches materials for [`MaterialWindow`].
    ///
    /// # Panics
    ///
    /// Panics if .env is missing or incomplete.
    fn load_materials(&mut self) {
        // Load data from api
        dotenv::dotenv().expect("No .env file found!");
        let api_key = env::var("API_KEY").expect("API Key missing!");

        if let Ok(materials) = ec3api::Ec3api::new(&api_key)
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

    fn render_material_cards(&self, ui: &mut eframe::egui::Ui, filter: &str) {
        for m in self
            .materials
            .iter()
            .filter(|mat| mat.title.to_lowercase().contains(filter))
        {
            ui.add_space(2.);

            ui.label(&m.title);
            ui.monospace(&m.gwp);
            // ui.label(RichText::from(&m.gwp).color(eframe::epaint::Color32::RED));

            // ui.hyperlink(&m.img_url); // removed for now
            ui.monospace(&m.country);
            ui.add_space(2.);

            ui.monospace(&m.category);
            ui.add_space(2.);

            ui.separator();
        }
    }
}

impl eframe::App for MaterialWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Top bar
        TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_visible(
                    self.materials_loaded,
                    egui::TextEdit::singleline(&mut self.search_input).hint_text("Search"),
                );
                ui.add_visible(
                    self.materials_loaded,
                    egui::Button::new("Switch visualization"),
                );
            })
        });
        // Bottom bar
        TopBottomPanel::bottom("bottom-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::global_dark_light_mode_switch(ui);
                ui.label(format!("{} materials", self.materials.len()));
            })
        });
        // Main panel
        CentralPanel::default().show(ctx, |ui| {
            if !self.materials_loaded {
                if ui.button("Load materials").clicked() {
                    self.load_materials();
                    self.materials_loaded = true;
                }
            }
            ui.add_space(4.);
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.render_material_cards(ui, &self.search_input.to_lowercase());
                });
        });
    }
}

#[derive(Clone)]
struct MaterialsData {
    title: String,
    gwp: String,
    // img_url: String,
    country: String,
    category: String,
}

fn main() -> Result<(), eframe::Error> {
    // init egui
    env_logger::init();
    let win_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(540., 800.)),
        resizable: false,
        follow_system_theme: true,
        ..Default::default()
    };

    eframe::run_native(
        "Materials",
        win_options,
        Box::new(|cc| Box::new(MaterialWindow::new(cc))),
    )
}

fn setup_custom_fonts(ctx: &eframe::egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../fonts/JetBrainsMonoNerdFontMono-BoldItalic.ttf"
        )),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
