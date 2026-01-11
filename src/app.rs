use core::f32;

use crate::{
    commands::{COMMAND_PREFIX, ChatCommand, CommandContext}, discord::{DiscordCommEvent, DiscordMessage}, utils
};
use egui::{Color32, Frame, RichText, ScrollArea, Style, TextEdit, text::LayoutJob};
use regex::Regex;
use tokio::sync::mpsc::{self, Receiver, Sender};

enum GuiMessage {
    User(String, String),
    Error(String),
    Generic(String)
}

pub struct App {
    main_frame: egui::Frame,
    messages: Vec<GuiMessage>,
    text_to_send: String,
    tx_to_dc: Sender<DiscordCommEvent>,
    rx_from_dc: Receiver<DiscordCommEvent>,
    token_regex: Regex,
    commands: Vec<ChatCommand>
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
            ).expect("Invalid regex pattern for token"),
            commands: vec![
                ChatCommand::one_alias("help")
                    .with_description("Shows a list of commands")
                    .with_handler(Self::cmd_help),
                ChatCommand::one_alias("login")
                    .with_description("Logs into Discord with the specified token")
                    .with_handler(Self::cmd_login)
            ]
        }
    }

    fn cmd_login(&mut self, ctx: CommandContext) {
        let token_arg = ctx.args.get(0);

        if token_arg.is_none() {
            self.add_message(GuiMessage::Error("Token not specified".to_string()));
            return;
        }

        let mut token = token_arg.unwrap().to_owned();

        if token == "env" {
            let env_token = std::env::var("DISCORD_TOKEN");

            match env_token {
                Ok(tok) => {
                    self.add_message(GuiMessage::Generic("Using token from env variables".to_string())); 
                    token = tok.to_owned() 
                },
                Err(_) => {
                    self.add_message(GuiMessage::Error("DISCORD_TOKEN env variable missing".to_string()));
                    return
                }
            };
        }

        self.add_message(GuiMessage::Generic("Logging in...".to_string()));
        self.transmit_to_dc(DiscordCommEvent::Login(token));
    }

    fn cmd_help(&mut self, _ctx: CommandContext) {
        // Rust makes me want to cry... WHY CAN'T I SIMPLY ITERATE ON A VECTOR ONCE AND CALL self.add_message() BUT I HAVE TO CREATE ANOTHER FRICKING VECTOR
        let mut msgs: Vec<GuiMessage> = Vec::new();

        for cmd in &self.commands {
            msgs.push(
                GuiMessage::Generic(format!(" {}: {}", cmd.aliases.join(","), cmd.description))
            );
        }

        self.add_message(GuiMessage::Generic("Available commands:".to_string()));

        for msg in msgs {
            self.add_message(msg);
        }
    }

    fn get_command(&self, alias: String) -> Option<ChatCommand> {
        for cmd in &self.commands {
            if cmd.aliases.contains(&alias) {
                return Some(cmd.clone());
            }
        }

        None
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

    fn clear_message(&mut self) {
        self.text_to_send = String::new();
    }

    fn process_command(&mut self, input: String) {
        let split: Vec<&str> = input.split_whitespace().collect();
        
        if split.is_empty() {
            return;
        }
        
        let alias = split[0];
        let args: Vec<String> = split[1..]
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        let cmd = self.get_command(alias.to_string());

        if let Some(cmd) = cmd {
            let ctx = CommandContext {
                alias: alias.to_string(),
                args,
            };

            cmd.execute(self, ctx);
        } else {
            self.add_message(GuiMessage::Error(format!("Unknown command '{}'", alias)));
        }
    }

    fn submit_message(&mut self) {
        let text = self.text_to_send.to_owned();

        if text.trim().is_empty() {
            return;
        }

        if text.starts_with(COMMAND_PREFIX) {
            let cmd_text = &text[COMMAND_PREFIX.len()..];

            self.clear_message();
            self.process_command(cmd_text.to_string());
            
            return;
        }
        
        if self.token_regex.is_match(&text) {
            self.add_message(GuiMessage::Error(
                "Your message was not sent, because it possibly contained Discord token.".to_string())
            );
            return;
        }

        self.transmit_to_dc(DiscordCommEvent::MessageSend(1459160075649286318, text.to_owned()));

        //self.add_message(GuiMessage::User("local".to_string(), text.to_string()));
        self.clear_message();
    }

    fn poll_discord_events(&mut self) {
        match self.rx_from_dc.try_recv() {
            Ok(event) => match event {
                DiscordCommEvent::Ready => {
                    self.add_message(GuiMessage::Generic("Logged in successfully".to_string()));
                },
                DiscordCommEvent::Error(text) => {
                    self.add_message(GuiMessage::Error(text));
                }
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
                            GuiMessage::Generic(text) => {
                                ui.label(text);
                            }
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
