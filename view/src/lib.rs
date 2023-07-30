extern crate shared;
use eframe::egui::{self, CentralPanel, RichText, ScrollArea, TopBottomPanel};
use shared::{SortBy, State};

#[no_mangle]
fn render_material_cards(state: &State, ui: &mut eframe::egui::Ui, filter: &str) {
    for m in state
        .materials
        .iter()
        .filter(|mat| mat.name.to_lowercase().contains(filter))
    {
        ui.add_space(2.);

        ui.label(&m.name);
        ui.monospace(&m.gwp.as_str());
        // ui.label(RichText::from(&m.gwp).color(eframe::epaint::Color32::RED));

        // ui.hyperlink(&m.img_url); // removed for nowlib
        ui.monospace(&m.manufacturer.country);
        ui.add_space(2.);

        ui.monospace(&m.category.description);
        ui.add_space(2.);

        ui.separator();
    }
}

#[no_mangle]
pub fn update_view(state: &mut State, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    // Top bar
    TopBottomPanel::top("top-bar").show(ctx, |ui| {
        ui.add_visible_ui(state.materials_loaded, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.search_input)
                        .hint_text("Search")
                        .desired_width(150.0),
                );
                ui.add(egui::Button::new("Switch visualization"));
            });
            ui.horizontal(|ui| {
                ui.label("sort by: ");
                if ui
                    .add(egui::RadioButton::new(
                        state.sort_by == SortBy::Name,
                        "name",
                    ))
                    .clicked()
                {
                    state.sort_by(SortBy::Name);
                }
                if ui
                    .add(egui::RadioButton::new(state.sort_by == SortBy::Gwp, "GWP"))
                    .clicked()
                {
                    state.sort_by(SortBy::Gwp);
                }
            })
        });
    });
    // Bottom bar
    TopBottomPanel::bottom("bottom-bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            egui::global_dark_light_mode_switch(ui);
            ui.label(format!("{} materials", state.materials.len()));
        });
    });
    // Main panel
    CentralPanel::default().show(ctx, |ui| {
        if !state.materials_loaded {
            if ui.button("Load materials").clicked() {
                state.load_materials();
                state.materials_loaded = true;
            }
        }
        ui.add_space(4.);
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                render_material_cards(state, ui, &state.search_input.to_lowercase());
            });
    });
}
