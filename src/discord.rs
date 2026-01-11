use std::{sync::Arc, thread::JoinHandle};

use serenity::{
    Client, FutureExt,
    all::{
        ChannelId, Context, CreateMessage, EventHandler, GatewayIntents, GuildId, Http, Message, Ready
    },
    async_trait,
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

                        http = Some(new_client.http.clone());

                        tokio::spawn(async move {
                            new_client.start().await.expect("Client error");
                        });                        
                    }
                    DiscordCommEvent::MessageSend(id, content) => {
                        let http = &http.to_owned().expect("Client not started");
                        ChannelId::new(id).say(http, content).await.expect("Failed to send message");
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
