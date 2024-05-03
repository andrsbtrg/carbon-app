#![windows_subsystem = "windows"]
extern crate shared;
extern crate view;

use eframe::
    egui::{self, ViewportBuilder}
;

struct Application {
    state: shared::State,
}

impl Application {
    fn new(cc: &eframe::CreationContext<'_>, api_key: Option<String>) -> Application {
        view::visuals::set_style(&cc.egui_ctx, view::visuals::Theme::dark());
        setup_custom_fonts(&cc.egui_ctx);

        Application {
            state: shared::State::new(api_key),
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        view::update_view(&mut self.state, ctx, _frame);
    }
}

// TODO: try this example https://github.com/rksm/hot-lib-reloader-rs/blob/master/examples/hot-egui/Cargo.toml
fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    // init egui
    let viewport = ViewportBuilder::default()
        .with_decorations(true)
        .with_title("Carbon")
        .with_resizable(true);
    let win_options = eframe::NativeOptions {
        viewport,
        run_and_return: true,
        ..Default::default()
    };
    setup_cache().unwrap_or_else(|e| {
        eprintln!("ERROR: unable to set up cache directory: {e}");
    });
    let api_key = get_api_key();
    eframe::run_native(
        "Carbon",
        win_options,
        Box::new(|cc| Box::new(Application::new(cc, api_key))),
    )
}

fn get_api_key() -> Option<String> {
    match std::fs::read_to_string(shared::settings::SettingsProvider::api_key_path()) {
        Ok(api_key) => Some(api_key),
        Err(_) => {
            println!("API key not found.");
            None
        }
    }
}

/// Creates .cache directory to store materials
fn setup_cache() -> Result<(), std::io::Error> {
    let dir = shared::settings::SettingsProvider::cache_dir();
    std::fs::create_dir_all(dir)
}

fn setup_custom_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../fonts/Inter.ttf")),
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
