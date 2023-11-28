extern crate shared;
use eframe::{
    egui::{self, CentralPanel, ComboBox, RichText, ScrollArea, TopBottomPanel},
    epaint::Color32,
};
use egui_plot::{Bar, BarChart, Plot};
use shared::{SortBy, State, Tabs};
use std::collections::BTreeSet;

const WHITE: Color32 = eframe::epaint::Color32::WHITE;

#[no_mangle]
/// Renders the materials available in the [State] state as a list view
fn render_material_cards(state: &State, ui: &mut eframe::egui::Ui, filter: &str) {
    for m in state
        .materials
        .iter()
        .filter(|mat| mat.name.to_lowercase().contains(filter))
        .filter(|mat| mat.category.name.contains(&state.selected_category))
    {
        ui.add_space(2.);

        if ui.style().visuals == egui::Visuals::dark() {
            ui.label(RichText::from(&m.name).color(WHITE));
        } else {
            ui.label(RichText::from(&m.name).color(Color32::BLACK));
        }
        ui.monospace(&m.gwp.as_str());

        // ui.hyperlink(&m.img_url); // removed for nowlib
        if let Some(c) = &m.manufacturer.country {
            ui.monospace(c);
        }
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
                add_tab(ui, state, Tabs::List);
                add_tab(ui, state, Tabs::Chart)
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
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.fetch_input);
            // TODO: should implement a search history
            // let response = ui.button("Open popup");
            // let popup_id = ui.make_persistent_id("my_unique_id");
            // if response.clicked() {
            //     ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            // }
            // let below = egui::AboveOrBelow::Below;
            // egui::popup::popup_above_or_below_widget(ui, popup_id, &response, below, |ui| {
            //     ui.set_min_width(200.0); // if you want to control the size
            //     ui.label("Some more info, or things you can select:");
            //     ui.label("…");
            // });
            if ui.button("Search").clicked() {
                state.search_materials();
            }
        });

        ui.collapsing("Advance search", |ui| {
            ui.label("Country:");
            ui.text_edit_singleline(&mut state.country);
            if ui.button("Advanced search").clicked() {
                // do something
                println!("Advance material search.");
            }
        });
        add_view_options(ui, state);

        add_category_filter(ui, state);

        ui.separator();

        ui.add_space(4.);

        match state.active_tab {
            shared::Tabs::Chart => render_material_chart(state, ui),

            shared::Tabs::List => {
                // if !state.materials_loaded {
                //     if ui.button("Load materials").clicked() {
                //         state.load_materials();
                //         // state.materials_loaded = true;
                //     };
                // };
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

fn add_tab(ui: &mut egui::Ui, state: &mut State, tab: Tabs) -> () {
    if ui
        .selectable_label(state.active_tab == tab, tab.to_string())
        .clicked()
    {
        state.active_tab = tab;
    };
}

fn add_category_filter(ui: &mut egui::Ui, state: &mut State) {
    if !state.materials_loaded {
        return;
    }
    let categories = state
        .materials
        .iter()
        .map(|mat| &mat.category.name)
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
            .filter(|(_, mat)| mat.category.name.contains(&state.selected_category))
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
    if !state.materials_loaded {
        return;
    }
    ui.collapsing("View", |ui| {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.search_input)
                    .hint_text("'material name'")
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
    });
}
fn fit_to_width(input: &String, len: usize) -> &str {
    if input.len() <= len {
        input
    } else {
        &input[..len]
    }
}
