use std::sync::Arc;

use serenity::{
    Client, FutureExt,
    all::{
        ChannelId, Context, CreateMessage, EventHandler, GatewayIntents, GuildId, Message, Ready,
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
    Ready,
    MessageSend(u64, String),
    MessageReceived(DiscordMessage),
}

pub struct DiscordHandler {
    tx: Sender<DiscordCommEvent>,
}

impl DiscordHandler {
    pub async fn transmit_to_gui(&self, event: DiscordCommEvent) {
        let tx = &self.tx;

        tx.send(event).await.unwrap_or_else(|err| {
            eprintln!("Failed to transmit event {}", err);
        });
    }

    pub async fn create_loop(
        token: String,
        tx: Sender<DiscordCommEvent>,
        mut rx: Receiver<DiscordCommEvent>,
    ) {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILDS
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(token, intents)
            .event_handler(Self { tx: tx })
            .await
            .expect("Client error");

        let http2 = client.http.clone();

        tokio::spawn(async move {
            client.start().await.expect("Client error");
        });

        loop {
            match rx.recv().await {
                Some(event) => match event {
                    DiscordCommEvent::MessageSend(id, content) => {
                        ChannelId::new(id).say(&http2, content).await.expect("shit");
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
