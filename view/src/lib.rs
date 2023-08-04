extern crate shared;
use std::collections::BTreeSet;

use eframe::{
    egui::{
        self,
        plot::{Bar, BarChart, Plot},
        CentralPanel, ComboBox, ScrollArea, TopBottomPanel,
    },
    epaint::Color32,
};
use shared::{SortBy, State, Tabs};

#[no_mangle]
/// Renders the materials available in the [State] state as a list view
fn render_material_cards(state: &State, ui: &mut eframe::egui::Ui, filter: &str) {
    for m in state
        .materials
        .iter()
        .filter(|mat| mat.name.to_lowercase().contains(filter))
        .filter(|mat| mat.category.description.contains(&state.selected_category))
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
/// Renders the view
pub fn update_view(state: &mut State, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    let loading = state.preload_data();
    // Top bar
    TopBottomPanel::top("top-bar").show(ctx, |ui| {
        ui.add_visible_ui(state.materials_loaded, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(state.active_tab == Tabs::List, "List")
                    .clicked()
                {
                    state.active_tab = Tabs::List;
                };
                if ui
                    .selectable_label(state.active_tab == Tabs::Chart, "Chart")
                    .clicked()
                {
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

        add_category_filter(ui, state);

        ui.separator();

        ui.add_space(4.);

        match state.active_tab {
            shared::Tabs::Chart => render_material_chart(state, ui),

            shared::Tabs::List => {
                if !state.materials_loaded {
                    if ui.button("Load materials").clicked() {
                        state.load_materials();
                        // state.materials_loaded = true;
                    };
                };
                if loading {
                    ui.spinner();
                }
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        render_material_cards(state, ui, &state.search_input.to_lowercase());
                    });
            }
        }
    });
}

fn add_category_filter(ui: &mut egui::Ui, state: &mut State) {
    let categories = state
        .materials
        .iter()
        .map(|mat| &mat.category.description)
        .collect::<BTreeSet<_>>();
    ui.horizontal(|ui| {
        ComboBox::from_id_source("category")
            // .wrap(true)
            .width(200.0)
            .selected_text(fit_to_width(&state.selected_category, 25))
            .show_ui(ui, |ui| {
                for cat in categories {
                    if ui
                        .selectable_label(state.selected_category == *cat, cat)
                        .clicked()
                    {
                        state.selected_category = cat.to_string();
                    }
                }
            });
        if ui.small_button("Clear").clicked() {
            state.selected_category = String::new();
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
            .enumerate()
            .filter(|(_, mat)| mat.category.description.contains(&state.selected_category))
            .map(|(i, mat)| {
                if mat.name.to_lowercase().contains(filter) {
                    Bar::new(i as f64, mat.gwp.value)
                        .name(&mat.name)
                        .fill(Color32::RED)
                } else {
                    Bar::new(i as f64, mat.gwp.value)
                        .name(&mat.name)
                        .fill(Color32::GRAY)
                }
            })
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
fn fit_to_width(input: &String, len: usize) -> &str {
    if input.len() <= len {
        input
    } else {
        &input[..len]
    }
}
