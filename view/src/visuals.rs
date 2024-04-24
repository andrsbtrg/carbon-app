use eframe::egui::{self};
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
