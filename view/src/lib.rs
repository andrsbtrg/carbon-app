extern crate shared;
use std::time::Duration;

use eframe::{
    egui::{self, CentralPanel, ComboBox, DragValue, RichText, ScrollArea, TopBottomPanel},
    epaint::Color32,
};
use egui_notify::Toast;
use egui_plot::{Bar, BarChart, Plot};
use shared::{
    project::{CmpResult, Project},
    SortBy, State, Tabs,
};

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
            add_tab(ui, state, Tabs::Category);
            add_tab(ui, state, Tabs::Calculate);
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
            shared::Tabs::Category => (),
            shared::Tabs::Calculate => calculate_page(state, ui),
        }
    });
    state.toasts.show(ctx);
}

fn calculate_page(state: &mut State, ui: &mut egui::Ui) {
    if state.project.is_none() {
        ui.label("Wow, such emptiness here!\nTry to start by adding a material from the list");
        return;
    }
    let mut grid_updated = false;
    let project = state.project.as_mut().unwrap();
    ui.add_space(10.);
    egui::Grid::new("my_grid")
        .num_columns(3)
        .max_col_width(200.)
        .min_row_height(40.)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Material");
            ui.label("Quantity");
            ui.label("Unit");
            ui.label("GWP (KgCO2e)");
            ui.label("-");
            ui.end_row();
            for (_i, comp) in project.components.iter_mut().enumerate() {
                ui.label(comp.material.get_name());
                let value = DragValue::new(&mut comp.quantity);
                if ui.add(value).changed() {
                    comp.calculate();
                    grid_updated = true;
                }
                ui.label(format!("{unit:?}", unit = &comp.material.get_unit().unit));
                ui.label(format!("{tots:.2}", tots = &comp.calculated));
                match comp.cmp_to_average() {
                    CmpResult::Smaller => ui
                        .label(RichText::new("↓").color(Color32::LIGHT_GREEN))
                        .on_hover_text(
                            "This material has a smaller GWP than the category average!",
                        ),
                    CmpResult::Greater => ui
                        .label(RichText::new("↑").color(Color32::LIGHT_RED))
                        .on_hover_text(
                            "This material has a greater GWP than the category average.",
                        ),
                    CmpResult::AlmostEqual => ui
                        .label(RichText::new("=").color(Color32::LIGHT_YELLOW))
                        .on_hover_text(
                            "This material has about the same GWP as the category average.",
                        ),
                };
                ui.end_row();
            } // end of iterating through components in project

            ComboBox::from_id_source("category-picker")
                .width(200.)
                .selected_text(fit_to_width(&state.selected_category, 25))
                .show_ui(ui, |ui| {
                    for cat in &state.loaded_categories {
                        if ui
                            .selectable_label(state.selected_category == *cat, cat)
                            .clicked()
                        {
                            project.add_generic_comp(&cat);
                        }
                    }
                });
            ui.label("Select a category to add a generic component");
            ui.end_row();
        });
    if grid_updated {
        project.calculate();
    };
    let total = match project.calculated_gwp > 1000. {
        true => RichText::new(format!(
            "Total GWP: {total:.2} T CO2e",
            total = &project.calculated_gwp / 1000.
        )),
        false => RichText::new(format!(
            "Total GWP: {total:.2} KgCO2e",
            total = &project.calculated_gwp
        )),
    }
    .color(Color32::WHITE);
    ui.add_space(4.);
    ui.label(total);
}

fn list_page(state: &mut State, ui: &mut egui::Ui, loading: bool) {
    add_filtering(ui, state);
    ui.separator();

    // render the material list to the left
    egui::SidePanel::left("list-materials")
        .resizable(true)
        .default_width(350.)
        .min_width(350.)
        .max_width(400.)
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
                    render_material_cards(state, ui, &state.filter_input.to_lowercase());
                });
        });
    // render the selected material in the central panel
    if let Some(selected) = &state.selected {
        ui.vertical_centered(|ui| {
            ui.heading(RichText::new(&selected.name).color(Color32::WHITE));
        });
        ui.add_space(2.0);
        ui.indent("general-selected", |ui| {
            ui.horizontal(|ui| {
                ui.label("Category: ");
                if ui
                    .selectable_label(false, &selected.category.display_name)
                    .on_hover_text(&selected.category.description)
                    .clicked()
                {
                    // go to category tab
                    state.toasts.info("Oops, let's forget about that");
                }
            });

            let avg_stat = state.category_stats.unwrap_or(0.);
            let cat_avg = RichText::new(format!(
                "Category average: {avg_stat:.2} {unit:?}",
                unit = selected.gwp.unit
            ));
            ui.label(cat_avg);
            let gwp = RichText::new(format!(
                "GWP: {gwp:.2} {unit:?}",
                gwp = selected.gwp.value,
                unit = selected.gwp.unit
            ));
            let color = match &avg_stat > &selected.gwp.value {
                true => Color32::LIGHT_GREEN,
                false => Color32::LIGHT_RED,
            };
            ui.label(gwp.color(color));
            if ui.button("Add to project →").clicked() {
                state.active_tab = shared::Tabs::Calculate;

                let copy = selected.clone();
                if let Some(project) = state.project.as_mut() {
                    project.add_component(copy, avg_stat);
                } else {
                    let mut p = Project::new();
                    p.add_component(copy, avg_stat);
                    state.project = Some(p);
                }
            }
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
    let cb = |t: &mut Toast| {
        //Callback for the toast
        t.set_closable(true)
            .set_duration(Some(Duration::from_millis((1000. * 3.5) as u64)));
    };
    egui::SidePanel::right("category-tree")
        .default_width(400.)
        .max_width(450.)
        .resizable(true)
        .show_inside(ui, |ui| categories_page(state, ui));
    ui.label("Search material by term");
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
            println!("Advanced material search.");
            state.fetch_materials(&state.fetch_input.clone());
        }
        if ui
            .button("Update db")
            .on_hover_text("This is a lengthy operation which downloads a new copy of EC3 materials locally for searching")
            .clicked()
        {
            let _ = shared::jobs::Runner::update_db(&state.api_key);
            cb(state.toasts.basic("Updating db"));
        }
        /*
        if ui.button("Load wood from cache").clicked() {
            &state.fetch_materials("Wood");
        }
        if ui.button("Save to db").clicked() {
            state.save_materials();
        }
        */
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
                    state.load_by_category(&tree.value.name);
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
                            state.load_by_category(name);
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
        ui.label("Search materials from a category");
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
            // calculate category stats
            state.category_stats = shared::material_db::get_category_stats(&m.category).ok();
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
    let filter = &state.filter_input;
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
        egui::TextEdit::singleline(&mut state.filter_input)
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
