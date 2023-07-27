use eframe::egui::{self, CentralPanel, ScrollArea, TopBottomPanel};
use shared::State;

fn render_material_cards(state: &State, ui: &mut eframe::egui::Ui, filter: &str) {
    for m in state
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

#[no_mangle]
pub fn update_view(state: &mut State, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    // Top bar
    TopBottomPanel::top("top-bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add_visible(
                state.materials_loaded,
                egui::TextEdit::singleline(&mut state.search_input).hint_text("Search"),
            );
            ui.add_visible(
                state.materials_loaded,
                egui::Button::new("Switch visualization"),
            );
        })
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
