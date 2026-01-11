use core::f32;

use crate::{
    discord::{DiscordCommEvent, DiscordMessage},
    utils,
};
use egui::{Color32, Frame, RichText, ScrollArea, Style, TextEdit, text::LayoutJob};
use regex::Regex;
use tokio::sync::mpsc::{self, Receiver, Sender};

enum GuiMessage {
    User(String, String),
    Error(String)
}

pub struct App {
    main_frame: egui::Frame,
    messages: Vec<GuiMessage>,
    text_to_send: String,
    tx_to_dc: Sender<DiscordCommEvent>,
    rx_from_dc: Receiver<DiscordCommEvent>,
    token_regex: Regex
}

impl App {
    pub fn new(tx_to_dc: Sender<DiscordCommEvent>, rx_from_dc: Receiver<DiscordCommEvent>) -> Self {
        Self {
            tx_to_dc: tx_to_dc,
            rx_from_dc: rx_from_dc,
            main_frame: Frame::new(),
            text_to_send: "".to_string(),
            messages: vec![],
            token_regex: Regex::new(
                r"[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{5,}\.[A-Za-z0-9_-]{16,}"
            ).expect("Invalid regex pattern for token")
        }
    }

    fn add_message(&mut self, msg: GuiMessage) {
        self.messages.push(msg);
    }

    fn transmit_to_dc(&mut self, event: DiscordCommEvent) {
        let tx = self.tx_to_dc.to_owned();

        tokio::spawn(async move {
            tx.send(event)
                .await
                .expect("Failed to send event to Discord thread");
        });
    }

    pub fn submit_message(&mut self) {
        let text = self.text_to_send.to_owned();

        if text.trim().is_empty() {
            return;
        }

        if self.token_regex.is_match(&text) {
            self.add_message(GuiMessage::Error(
                "Your message was not sent, because it contained a possible Discord token.".to_string())
            );
            return;
        }

        self.transmit_to_dc(DiscordCommEvent::MessageSend(1459160075649286318, text.to_owned()));

        self.add_message(GuiMessage::User("local".to_string(), text.to_string()));

        self.text_to_send = String::new();
    }

    fn poll_discord_events(&mut self) {
        match self.rx_from_dc.try_recv() {
            Ok(event) => match event {
                DiscordCommEvent::MessageReceived(msg) => {
                    self.add_message(GuiMessage::User(
                        msg.author.display_name().to_string(), 
                        msg.content
                    ));
                }
                _ => (),
            },
            Err(_) => (),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_discord_events();

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
                let msg_input =
                    TextEdit::singleline(&mut self.text_to_send).hint_text("Type your message...");
                let msg_input_resp = ui.add_sized(ui.available_size(),msg_input);

                if utils::ui::input_submitted(&msg_input_resp, &ui) {
                    self.submit_message();
                }

                if ui.input(|inp| inp.key_down(egui::Key::Slash)) {
                    msg_input_resp.request_focus();
                }
        });

        egui::CentralPanel::default()
            .frame(self.main_frame)
            .show(ctx, |ui| {
                let msgs = &self.messages;

                let chat_scroll = ScrollArea::vertical().auto_shrink([false, false]);
                
                chat_scroll.show_rows(ui, 10.0, msgs.len(), |ui, row_range| {
                    for i in row_range {
                        let msg = &msgs[i];

                        match msg {
                            GuiMessage::User(name, text) => {
                                ui.label(RichText::new(format!("{}: {}", name, text)).color(Color32::WHITE));
                            },
                            GuiMessage::Error(text) => {
                                ui.label(RichText::new(text).color(Color32::RED));
                            }
                        }
                    }
                });

            });

        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.5]
    }
}
