extern crate shared;
use eframe::{
    egui::{self, CentralPanel, ComboBox, RichText, ScrollArea, TopBottomPanel},
    epaint::Color32,
};
use egui_plot::{Bar, BarChart, Plot};
use shared::{SortBy, State, Tabs};

/// Renders the view
#[no_mangle]
pub fn update_view(state: &mut State, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    let loading = state.preload_data();
    // Top bar
    TopBottomPanel::top("top-bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            add_tab(ui, state, Tabs::Search);
            add_tab(ui, state, Tabs::List);
            add_tab(ui, state, Tabs::Chart);
            // add_tab(ui, state, Tabs::Categories);
        });
    });
    // Bottom bar
    TopBottomPanel::bottom("bottom-bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // egui::global_dark_light_mode_switch(ui);
            ui.label(format!("{} materials", state.materials.len()));
        });
    });
    // Main panel
    // let frame = egui::Frame {
    //     inner_margin: egui::Margin {
    //         left: 20.,
    //         right: 10.,
    //         top: 5.,
    //         bottom: 5.,
    //     },
    //     ..Default::default()
    // };
    CentralPanel::default().show(ctx, |ui| {
        ui.add_space(4.);
        match state.active_tab {
            shared::Tabs::Search => search_page(state, ui),
            shared::Tabs::Chart => chart_page(state, ui),
            shared::Tabs::List => list_page(state, ui, loading),
            shared::Tabs::Categories => (),
        }
    });
}

fn list_page(state: &mut State, ui: &mut egui::Ui, loading: bool) {
    add_filtering(ui, state);
    ui.separator();

    // render the material list to the left
    egui::SidePanel::left("list-materials")
        .resizable(true)
        .default_width(350.0)
        .max_width(400.0)
        .show_inside(ui, |panel_ui| {
            if loading {
                panel_ui.vertical_centered_justified(|ui| {
                    ui.label("Loading...");
                    // ui.spinner();
                });
                return;
            }
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(panel_ui, |ui| {
                    render_material_cards(state, ui, &state.search_input.to_lowercase());
                });
        });
    // render the selected material in the central panel
    if let Some(selected) = &state.selected {
        ui.vertical_centered(|ui| {
            ui.heading(&selected.name);
        });
        ui.separator();
        ScrollArea::vertical().show(ui, |ui| {
            ui.indent("more", |ui| {
                ui.add_space(2.0);
                ui.label("Description: ");
                ui.label(&selected.description);
                ui.add_space(2.0);
            });
        });
    }
}

fn chart_page(state: &mut State, ui: &mut egui::Ui) {
    add_filtering(ui, state);
    ui.separator();
    render_material_chart(state, ui);
}

fn add_filtering(ui: &mut egui::Ui, state: &mut State) {
    if !state.materials_loaded {
        return;
    }

    ui.horizontal(|ui| {
        add_view_options(ui, state);
        add_category_filter(ui, state);
    });
}

fn search_page(state: &mut State, ui: &mut egui::Ui) {
    egui::SidePanel::right("category-tree")
        .default_width(400.)
        .max_width(450.)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.label("Search materials from a category");
            categories_page(state, ui)
        });
    ui.heading("Search the EC3 Database");
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.fetch_input);
        if ui
            .button("Search")
            .on_hover_text("Type a material name to search in EC3")
            .clicked()
        {
            if !state.fetch_input.is_empty() {
                state.fetch_materials_from_input();
                state.active_tab = shared::Tabs::List;
            }
        }
    });
    ui.collapsing("More options", |ui| {
        ui.label("Country:");
        ui.text_edit_singleline(&mut state.country);
        if ui.button("Search").clicked() {
            // do something
            println!("Advance material search.");
        }
        if ui.button("Update db").clicked() {
            // TODO: Make async
            let _ = shared::jobs::Runner::update_db(&state.api_key);
        }
    });
}

/// Render recursively nodes in [shared::CategoriesTree]
fn render_tree(ui: &mut egui::Ui, tree: &shared::CategoriesTree, state: &mut State) {
    if let Some(subcategories) = &tree.children {
        if subcategories.len() == 0 {
            ui.horizontal(|ui| {
                ui.label(&tree.value.name);
                if ui
                    .small_button("→")
                    .on_hover_text(format!("Search {}", tree.value.name))
                    .clicked()
                {
                    // use the callback function here
                    state.search_materials(&tree.value.name);
                    state.active_tab = shared::Tabs::List;
                };
            });
        } else {
            for v in subcategories {
                let name = &v.value.name.clone();
                ui.horizontal(|ui| {
                    let coll = ui.collapsing(name, |ui| {
                        render_tree(ui, &v, state);
                    });
                    if !coll.fully_open() {
                        if ui
                            .small_button("→")
                            .on_hover_text(format!("Search {name}"))
                            .clicked()
                        {
                            // use the callback function here
                            state.search_materials(name);
                            state.active_tab = shared::Tabs::List;
                        }
                    }
                });
            }
        }
    }
}

/// Lazy loads and renders [shared::CategoriesTree]
fn categories_page(state: &mut State, ui: &mut egui::Ui) {
    if state.preload_categories() {
        ui.vertical_centered_justified(|ui| {
            ui.label("Loading...");
            // ui.spinner();
        });
        return;
    }

    if let Some(categories) = state.categories.clone() {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                render_tree(ui, &categories, state);
            });
    } else {
        state.load_categories();
    }
}

/// Renders the materials available in the [State] state as a list view
fn render_material_cards(state: &mut State, ui: &mut eframe::egui::Ui, filter: &str) {
    for m in state
        .materials
        .iter()
        .filter(|mat| mat.name.to_lowercase().contains(filter))
        .filter(|mat| {
            if &state.selected_category != "" {
                mat.category.name == state.selected_category
            } else {
                true
            }
        })
    {
        ui.add_space(2.);
        if ui
            .selectable_label(
                false,
                RichText::new(&m.name).heading().color(Color32::WHITE),
            )
            .clicked()
        {
            state.selected = Some(m.clone());
        }
        ui.monospace(&m.gwp.as_str());
        if let Some(c) = &m.manufacturer.country {
            ui.monospace(format!("Country: {c}"));
        }
        ui.add_space(2.);
        ui.label(&m.category.name)
            .on_hover_text(&m.category.description);
        ui.add_space(2.);
        ui.separator();
    }
}

/// Short way of adding a tab that is connected to a [Tabs] enum in [State]
fn add_tab(ui: &mut egui::Ui, state: &mut State, tab: Tabs) -> () {
    if ui
        .selectable_label(state.active_tab == tab, format!("   {tab}   "))
        .clicked()
    {
        state.active_tab = tab;
    };
}

fn add_category_filter(ui: &mut egui::Ui, state: &mut State) {
    ComboBox::from_id_source("category")
        .width(200.0)
        .selected_text(fit_to_width(&state.selected_category, 25))
        .show_ui(ui, |ui| {
            for cat in &state.loaded_categories {
                if ui
                    .selectable_label(state.selected_category == *cat, cat)
                    .clicked()
                {
                    state.selected_category = cat.to_string();
                }
            }
        });
    if !state.selected_category.is_empty() {
        if ui.small_button("Clear").clicked() {
            state.selected_category = String::new();
        }
    }
}

/// Renders the materials available in the [State] state as a chart
fn render_material_chart(state: &mut State, ui: &mut egui::Ui) {
    let filter = &state.search_input;
    let primary = ui.visuals().selection.bg_fill;
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
                        .fill(primary)
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
    ui.add(
        egui::TextEdit::singleline(&mut state.search_input)
            .hint_text("filter by name")
            .desired_width(200.0),
    );
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
}
fn fit_to_width(input: &String, len: usize) -> &str {
    if input.len() <= len {
        input
    } else {
        &input[..len]
    }
}
