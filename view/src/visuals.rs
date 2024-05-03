use eframe::egui::{self, Color32};
const WIDTH: f32 = 400.;
const INNER_MARGIN: f32 = 6.;
pub struct Panels {}
impl Panels {
    pub fn left() -> egui::SidePanel {
        let frame = egui::Frame::default().inner_margin(INNER_MARGIN);

        egui::SidePanel::left("left-panel")
            .default_width(WIDTH)
            .resizable(true)
            .frame(frame)
    }
}

pub struct Theme {
    pub dark: bool,
    pub bg_color: Color32,
    pub panel_bg_color: Color32,
    pub inactive_bg_fill: Color32,
    pub hovered_color: Color32,
    pub highlight_color: Color32,
    pub bg_stroke_color: Color32,// from figma. separator lines, panel lines, etc
    pub default: Color32,
    pub subdued: Color32,
    pub strong: Color32,
    pub floating_color: Color32,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            dark: false,
            bg_color: Color32::from_gray(255),
            panel_bg_color: Color32::from_gray(248),
            inactive_bg_fill: Color32::from_rgb(250, 250, 250),
            hovered_color: Color32::from_gray(230),
            highlight_color: Color32::from_rgb(90, 129, 255),
            bg_stroke_color: Color32::from_gray(190),
            default: Color32::from_gray(60),
            subdued: Color32::from_gray(80),
            strong: Color32::from_rgb(1, 1, 1),
            floating_color: Color32::from_gray(195),
        }
    }
    pub fn dark() -> Self {
        Self {
            dark: true,
            bg_color: Color32::BLACK,
            panel_bg_color: Color32::from_rgb(13, 16, 17),
            inactive_bg_fill: Color32::from_rgb(5, 6, 7),
            hovered_color: Color32::from_gray(64),
            highlight_color: Color32::from_rgb(90, 129, 255),
            bg_stroke_color: Color32::from_gray(30),
            default: Color32::from_rgb(202, 216, 222),
            subdued: Color32::from_rgb(108, 121, 127),
            strong: Color32::from_rgb(249, 249, 249),
            floating_color: Color32::from_gray(35),
        }
    }
}

pub fn set_style(ctx: &egui::Context, theme: Theme) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let base_style = match theme.dark {
        true => egui::Visuals::dark(),
        false => egui::Visuals::light(),
    };
    let mut egui_style = egui::Style {
        visuals: base_style,
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
    egui_style.visuals.extreme_bg_color = theme.bg_color;

    let panel_bg_color = theme.panel_bg_color;

    egui_style.visuals.widgets.noninteractive.weak_bg_fill = panel_bg_color;
    egui_style.visuals.widgets.noninteractive.bg_fill = panel_bg_color;

    egui_style.visuals.button_frame = true;
    egui_style.visuals.widgets.inactive.weak_bg_fill = Default::default(); // Buttons have no background color when inactive
    egui_style.visuals.widgets.inactive.bg_fill = theme.inactive_bg_fill;
    // Fill of unchecked radio buttons, checkboxes, etc. Must be brigher than the background floating_color

    {
        // Background colors for buttons (menu buttons, blueprint buttons, etc) when hovered or clicked:
        // let hovered_color = get_aliased_color(&json, "{Alias.Color.Action.Hovered.value}");
        let hovered_color = theme.hovered_color;
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

    let highlight_color = theme.highlight_color;
    egui_style.visuals.selection.bg_fill = highlight_color;

    egui_style.visuals.widgets.noninteractive.bg_stroke.color = theme.bg_stroke_color; // from figma. separator lines, panel lines, etc

    let default = theme.default;
    let subdued = theme.subdued;
    let strong = theme.strong;
    egui_style.visuals.widgets.noninteractive.fg_stroke.color = subdued; // non-interactive text
    egui_style.visuals.widgets.inactive.fg_stroke.color = default; // button text
    egui_style.visuals.widgets.active.fg_stroke.color = strong; // strong text and active button text

    egui_style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    egui_style.visuals.window_shadow = egui::epaint::Shadow::NONE;

    let floating_color = theme.floating_color;
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
}
