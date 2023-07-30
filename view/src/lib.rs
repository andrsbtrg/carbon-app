extern crate shared;
use eframe::egui::{
    self,
    plot::{Bar, BarChart, Plot},
    CentralPanel, ScrollArea, TopBottomPanel,
};
use shared::{SortBy, State, Tabs};

#[no_mangle]
/// Renders the materials available in the [State] state as a list view
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
                if ui.button("List").clicked() {
                    state.active_tab = Tabs::List;
                };
                if ui.button("Chart").clicked() {
                    state.active_tab = Tabs::Chart;
                }
            });
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
        add_view_options(ui, state);

        ui.separator();

        ui.add_space(4.);

        match state.active_tab {
            shared::Tabs::Chart => render_material_chart(state, ui),

            shared::Tabs::List => {
                if !state.materials_loaded {
                    if ui.button("Load materials").clicked() {
                        state.load_materials();
                        state.materials_loaded = true;
                    };
                };
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        render_material_cards(state, ui, &state.search_input.to_lowercase());
                    });
            }
        }
    });
}

/// Renders the materials available in the [State] state as a chart
fn render_material_chart(state: &mut State, ui: &mut egui::Ui) {
    let filter = &state.search_input;
    let chart = BarChart::new(
        state
            .materials
            .iter()
            .filter(|mat| mat.name.to_lowercase().contains(filter))
            .enumerate()
            .map(|(i, mat)| Bar::new(i as f64, mat.gwp.value))
            .collect(),
    );

    Plot::new("plot").show(ui, |plot_ui| {
        plot_ui.bar_chart(chart);
    });
}

/// Adds a search input and sorting options to the UI
fn add_view_options(ui: &mut egui::Ui, state: &mut State) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.search_input)
                .hint_text("Search")
                .desired_width(200.0),
        );
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
    });
}
