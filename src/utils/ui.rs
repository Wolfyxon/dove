use egui::{Align, FontSelection, Response, RichText, Style, Ui, text::LayoutJob};

pub fn input_submitted(resp: &Response, ui: &Ui) -> bool {
    resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}

pub fn combine_rich_text(texts: Vec<impl Into<RichText>>) -> LayoutJob {
    let style = Style::default();
    let mut layout_job = LayoutJob::default();

    for text in texts {
        text.into()
            .append_to(
                &mut layout_job, 
                &style, 
                FontSelection::Default, 
                Align::Min
            );
    }

    layout_job
}