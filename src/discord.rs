use std::{sync::Arc};

use serenity::{
    Client,
    all::{
        ChannelId, Context, EventHandler, GatewayError, GatewayIntents, GuildId, Http, Message, Ready
    },
    async_trait,
};
use tokio::sync::{Mutex, mpsc::{Receiver, Sender}};

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

pub struct DiscordManager {
    tx: Sender<DiscordCommEvent>
}

impl DiscordManager {
    pub fn new(tx: Sender<DiscordCommEvent>) -> Self {
        Self {
            tx: tx
        }
    }

    pub async fn start(
        &self,
        mut rx: Receiver<DiscordCommEvent>,
    ) {
        let http_mutex: Arc<Mutex<Option<Arc<Http>>>> = Arc::new(Mutex::new(None));
        let tx = self.tx.clone();

        loop {
            match rx.recv().await {
                Some(event) => match event {
                    DiscordCommEvent::Login(token) => {
                        let mut new_client = Self::new_client(token, tx.clone()).await;
                        let tx2 = tx.to_owned();

                        let http_mutex = http_mutex.clone();
                        let http_mutex2 = http_mutex.clone();
                        
                        let mut http = http_mutex.lock().await;
                        *http = Some(new_client.http.clone());

                        tokio::spawn(async move {
                            let client_res = new_client.start().await;

                            let mut http_mutex = http_mutex2.lock().await;
                            *http_mutex = None;

                            if let Err(e) = client_res {
                                let mut error_string = format!("{:?}: {}", e, e);

                                if let serenity::Error::Gateway(e) = e {
                                    if matches!(e, GatewayError::InvalidAuthentication) {
                                        error_string = "Invalid token".to_string();
                                    }
                                }

                                let event = DiscordCommEvent::Error(error_string);
                                tx2.send(event).await.expect("Error transmission failed");
                            }
     
                        });
                    }
                    DiscordCommEvent::MessageSend(id, content) => {
                        let http = http_mutex.lock().await;
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

    async fn new_client(token: String, tx: Sender<DiscordCommEvent>) -> Client {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILDS
            | GatewayIntents::MESSAGE_CONTENT;

        let client = Client::builder(token, intents)
            .event_handler(DiscordHandler { tx: tx })
            .await
            .expect("Client error");

        client
    }
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
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, _ctx: Context, msg: Message) {
        println!("Received {}", &msg.content);

        self.transmit_to_gui(DiscordCommEvent::MessageReceived(msg))
            .await;
    }

    async fn ready(&self, _ctx: Context, _ready: Ready) {
        println!("Discord ready")
    }

    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {
        println!("Discord cache ready");
        self.transmit_to_gui(DiscordCommEvent::Ready).await;
    }
}
