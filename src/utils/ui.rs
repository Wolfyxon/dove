use egui::{Response, Ui};

pub fn input_submitted(resp: &Response, ui: &Ui) -> bool {
    resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}
