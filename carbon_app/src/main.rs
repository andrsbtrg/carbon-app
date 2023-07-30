extern crate shared;

#[cfg(feature = "hot_reload_libs")]
extern crate hot_reload_lib;

#[cfg(not(feature = "hot_reload_libs"))]
extern crate view;

use eframe::{egui, epaint::Vec2};

#[cfg(feature = "hot_reload_libs")]
use hot_reload_lib::HotReloadLib;

use std::env;

#[cfg(feature = "hot_reload_libs")]
struct HotReloadLibs {
    view: HotReloadLib,
}

#[cfg(feature = "hot_reload_libs")]
impl HotReloadLibs {
    fn new(hot_reload_libs_folder: &str) -> Self {
        Self {
            view: HotReloadLib::new(hot_reload_libs_folder, "view"),
        }
    }

    fn update_libs(&mut self) {
        if self.view.update() {
            println!("Reloaded view lib");
        }
    }
}

struct Application {
    state: shared::State,

    #[cfg(feature = "hot_reload_libs")]
    libs: HotReloadLibs,
}

impl Application {
    fn new(
        cc: &eframe::CreationContext<'_>,
        _hot_reload_libs_folder: &str,
        api_key: String,
    ) -> Application {
        setup_custom_fonts(&cc.egui_ctx);

        Application {
            state: shared::State::new(api_key),

            #[cfg(feature = "hot_reload_libs")]
            libs: HotReloadLibs::new(_hot_reload_libs_folder),
        }
    }
}

#[cfg(not(feature = "hot_reload_libs"))]
impl eframe::App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        view::update_view(&mut self.state, ctx, _frame);
    }
}

// TODO: try this example https://github.com/rksm/hot-lib-reloader-rs/blob/master/examples/hot-egui/Cargo.toml

#[cfg(feature = "hot_reload_libs")]
impl eframe::App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|input_state| {
            if input_state.key_down(egui::Key::R) {
                self.libs.update_libs();
            }
        });

        self.libs
            .view
            .load_symbol::<fn(&shared::State, &eframe::egui::Context, &mut eframe::Frame)>(
                "update_view",
            )(&self.state, ctx, _frame);
    }
}

fn main() -> Result<(), eframe::Error> {
    let libraries_path = "target/debug";

    // read environment
    dotenv::dotenv().expect("No .env file found!");
    let api_key = env::var("API_KEY").expect("API Key missing!");

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
        Box::new(|cc| Box::new(Application::new(cc, libraries_path, api_key))),
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
