use crate::{
    discord::{DiscordCommEvent, DiscordMessage},
    utils,
};
use egui::{Color32, Frame, RichText, ScrollArea, TextEdit};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct Message {
    username: String,
    text: String,
}

pub struct App {
    main_frame: egui::Frame,
    messages: Vec<Message>,
    text_to_send: String,
    tx_to_dc: Sender<DiscordCommEvent>,
    rx_from_dc: Receiver<DiscordCommEvent>,
}

impl App {
    pub fn new(tx_to_dc: Sender<DiscordCommEvent>, rx_from_dc: Receiver<DiscordCommEvent>) -> Self {
        Self {
            tx_to_dc: tx_to_dc,
            rx_from_dc: rx_from_dc,
            main_frame: Frame::new(),
            text_to_send: "".to_string(),
            messages: vec![],
        }
    }

    fn transmit_to_dc(&mut self, event: DiscordCommEvent) {
        let tx = self.tx_to_dc.to_owned();

        tokio::spawn(async move {
            tx.send(event)
                .await
                .expect("Failed to send event to Discord thread");
        });
    }

    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn submit_message(&mut self) {
        let text = self.text_to_send.to_owned();

        if text.trim().is_empty() {
            return;
        }

        self.transmit_to_dc(DiscordCommEvent::MessageSend(1459160075649286318, text));

        println!("sent");

        self.text_to_send = String::new();

        /*self.add_message(Message {
            username: "Local".to_string(),
            text: text.to_owned()
        });*/
    }

    fn poll_discord_events(&mut self) {
        match self.rx_from_dc.try_recv() {
            Ok(event) => match event {
                DiscordCommEvent::MessageReceived(msg) => {
                    println!("recv");
                    self.add_message(Message {
                        username: msg.author.name,
                        text: msg.content,
                    });
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

        egui::CentralPanel::default()
            .frame(self.main_frame)
            .show(ctx, |ui| {
                let msgs = &self.messages;

                ScrollArea::vertical().show_rows(ui, 10.0, msgs.len(), |ui, row_range| {
                    for i in row_range {
                        let msg = &msgs[i];
                        let label_text = format!("{}: {}", msg.username, msg.text);

                        ui.label(RichText::new(label_text).color(Color32::WHITE));
                    }
                });

                let msg_input =
                    TextEdit::singleline(&mut self.text_to_send).hint_text("Type your message...");
                let msg_input_resp = ui.add(msg_input);

                if utils::ui::input_submitted(&msg_input_resp, &ui) {
                    self.submit_message();
                }
            });

        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.5]
    }
}
