use std::{sync::Arc, thread::JoinHandle};

use serenity::{
    Client, FutureExt,
    all::{
        ChannelId, Context, CreateMessage, EventHandler, GatewayIntents, GuildId, Http, Message, Ready
    },
    async_trait, futures::TryFutureExt,
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::sleep;

use crate::utils::comm::MPSCChannel;

type OnMessageHandler = fn(msg: Message) -> ();

pub type DiscordMessage = serenity::all::Message;

#[derive(Debug)]
pub enum DiscordCommEvent {
    // GUI -> Discord
    Login(String),
    MessageSend(u64, String),
    // Discord -> GUI
    Ready,
    Error(String),
    MessageReceived(DiscordMessage),
}

pub struct DiscordHandler {
    tx: Sender<DiscordCommEvent>,
}

impl DiscordHandler {
    async fn transmit_to_gui(&self, event: DiscordCommEvent) {
        let tx = &self.tx;

        tx.send(event).await.unwrap_or_else(|err| {
            eprintln!("Failed to transmit event {}", err);
        });
    }

    async fn create_client(token: String, tx: Sender<DiscordCommEvent>) -> Client {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILDS
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(token, intents)
            .event_handler(Self { tx: tx })
            .await
            .expect("Client error");

        client
    }

    pub async fn create_loop(
        tx: Sender<DiscordCommEvent>,
        mut rx: Receiver<DiscordCommEvent>,
    ) {
        let mut http: Option<Arc<Http>> = None;

        loop {
            match rx.recv().await {
                Some(event) => match event {
                    DiscordCommEvent::Login(token) => {
                        let mut new_client = Self::create_client(token, tx.clone()).await;
                        let tx2 = tx.to_owned();

                        http = Some(new_client.http.clone());

                        tokio::spawn(async move {
                            let client_res = new_client.start().await;

                            if let Err(e) = client_res {
                                let event = DiscordCommEvent::Error(format!("Connection aborted: {}", e));
                                tx2.send(event).await.expect("Error transmission failed");
                            }
     
                        });
                    }
                    DiscordCommEvent::MessageSend(id, content) => {
                        let http = &http.to_owned();
                        let tx = tx.to_owned();

                        if let Some(http) = &http.to_owned() {
                            let msg_res = ChannelId::new(id).say(http, content).await;

                            if let Err(e) = msg_res {
                                tx.send(DiscordCommEvent::Error(format!("Unable to send message: {}", e.to_string()))).await.expect("Msg error transmission failed");
                            }
                        } else {
                            tx.send(DiscordCommEvent::Error("Not logged in".to_string())).await.expect("HTTP missing transmission failed");
                        }
                    }
                    _ => (),
                },
                None => (),
            };
        }
    }
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("Received {}", &msg.content);

        self.transmit_to_gui(DiscordCommEvent::MessageReceived(msg))
            .await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Discord ready")
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Discord cache ready");
        self.transmit_to_gui(DiscordCommEvent::Ready).await;
    }
}
