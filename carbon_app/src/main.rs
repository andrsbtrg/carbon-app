extern crate shared;

#[cfg(feature = "hot_reload_libs")]
extern crate hot_reload_lib;

#[cfg(not(feature = "hot_reload_libs"))]
extern crate view;

use eframe::{
    egui::{self, ViewportBuilder},
    epaint::Color32,
};

#[cfg(feature = "hot_reload_libs")]
use hot_reload_lib::HotReloadLib;

use std::{env, path::Path};

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

    // create cache dir
    setup_cache();

    // init egui
    env_logger::init();
    let viewport = ViewportBuilder::default()
        .with_decorations(true)
        .with_resizable(true);

    let win_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Materials",
        win_options,
        Box::new(|cc| Box::new(Application::new(cc, libraries_path, api_key))),
    )
}

fn setup_cache() -> () {
    let dir = Path::new(".cache");
    if dir.exists() {
        println!("Cache directory set")
    } else {
        std::fs::create_dir(dir).expect("Unable to create cache dir");
        println!("Cache directory created")
    }
}

fn setup_custom_fonts(ctx: &eframe::egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut egui_style = egui::Style {
        visuals: egui::Visuals::dark(),
        ..Default::default()
    };
    for text_style in [
        egui::TextStyle::Body,
        egui::TextStyle::Monospace,
        egui::TextStyle::Button,
    ] {
        egui_style.text_styles.get_mut(&text_style).unwrap().size = 16.0;
    }
    egui_style
        .text_styles
        .get_mut(&egui::TextStyle::Heading)
        .unwrap()
        .size = 16.0;
    egui_style.spacing.interact_size.y = 15.0;
    egui_style.visuals.extreme_bg_color = egui::Color32::BLACK;

    let panel_bg_color = Color32::from_rgb(13, 16, 17);

    egui_style.visuals.widgets.noninteractive.weak_bg_fill = panel_bg_color;
    egui_style.visuals.widgets.noninteractive.bg_fill = panel_bg_color;

    egui_style.visuals.button_frame = true;
    egui_style.visuals.widgets.inactive.weak_bg_fill = Default::default(); // Buttons have no background color when inactive
    egui_style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(5, 6, 7);
    // Fill of unchecked radio buttons, checkboxes, etc. Must be brigher than the background floating_color

    {
        // Background colors for buttons (menu buttons, blueprint buttons, etc) when hovered or clicked:
        // let hovered_color = get_aliased_color(&json, "{Alias.Color.Action.Hovered.value}");
        let hovered_color = Color32::from_gray(64); // TODO(emilk): change the content of the design_tokens.json origin instead
        egui_style.visuals.widgets.hovered.weak_bg_fill = hovered_color;
        egui_style.visuals.widgets.hovered.bg_fill = hovered_color;
        egui_style.visuals.widgets.active.weak_bg_fill = hovered_color;
        egui_style.visuals.widgets.active.bg_fill = hovered_color;
        egui_style.visuals.widgets.open.weak_bg_fill = hovered_color;
        egui_style.visuals.widgets.open.bg_fill = hovered_color;
    }

    {
        // Turn off strokes around buttons:
        egui_style.visuals.widgets.inactive.bg_stroke = Default::default();
        egui_style.visuals.widgets.hovered.bg_stroke = Default::default();
        egui_style.visuals.widgets.active.bg_stroke = Default::default();
        egui_style.visuals.widgets.open.bg_stroke = Default::default();
    }

    {
        egui_style.visuals.widgets.hovered.expansion = 2.0;
        egui_style.visuals.widgets.active.expansion = 2.0;
        egui_style.visuals.widgets.open.expansion = 2.0;
    }

    let highlight_color = Color32::from_rgb(90, 129, 255);
    egui_style.visuals.selection.bg_fill = highlight_color;

    egui_style.visuals.widgets.noninteractive.bg_stroke.color = Color32::from_gray(30); // from figma. separator lines, panel lines, etc

    let default = Color32::from_rgb(202, 216, 222);
    let subdued = Color32::from_rgb(108, 121, 127);
    let strong = Color32::from_rgb(1, 1, 1);
    egui_style.visuals.widgets.noninteractive.fg_stroke.color = subdued; // non-interactive text
    egui_style.visuals.widgets.inactive.fg_stroke.color = default; // button text
    egui_style.visuals.widgets.active.fg_stroke.color = strong; // strong text and active button text

    egui_style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    egui_style.visuals.window_shadow = egui::epaint::Shadow::NONE;

    let floating_color = Color32::from_gray(35);
    egui_style.visuals.window_fill = floating_color; // tooltips and menus
    egui_style.visuals.window_stroke = egui::Stroke::NONE;
    egui_style.visuals.panel_fill = panel_bg_color;

    egui_style.visuals.window_rounding = 12.0.into();
    egui_style.visuals.menu_rounding = 12.0.into();
    let small_rounding = 4.0.into();
    egui_style.visuals.widgets.noninteractive.rounding = small_rounding;
    egui_style.visuals.widgets.inactive.rounding = small_rounding;
    egui_style.visuals.widgets.hovered.rounding = small_rounding;
    egui_style.visuals.widgets.active.rounding = small_rounding;
    egui_style.visuals.widgets.open.rounding = small_rounding;

    egui_style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    // egui_style.spacing.menu_margin = crate::ReUi::view_padding().into();

    // avoid some visual glitches with the default non-zero value
    egui_style.visuals.clip_rect_margin = 0.0;

    // Add stripes to grids and tables?
    egui_style.visuals.striped = false;
    egui_style.visuals.indent_has_left_vline = false;
    egui_style.spacing.button_padding = egui::Vec2::new(1.0, 0.0); // Makes the icons in the blueprint panel align
    egui_style.spacing.indent = 14.0; // From figma

    egui_style.spacing.combo_width = 8.0; // minimum width of ComboBox - keep them small, with the down-arrow close.

    egui_style.spacing.scroll.bar_inner_margin = 2.0;
    egui_style.spacing.scroll.bar_width = 6.0;
    egui_style.spacing.scroll.bar_outer_margin = 2.0;

    // don't color hyperlinks #2733
    egui_style.visuals.hyperlink_color = default;

    egui_style.visuals.image_loading_spinners = false;

    ctx.set_style(egui_style);

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
