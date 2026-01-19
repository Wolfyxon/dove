use std::sync::Arc;

use serenity::{
    Client,
    all::{
        ChannelId, Context, EventHandler, GatewayError, GatewayIntents, GuildId, Http, Message,
        Ready, ShardManager,
    },
    async_trait,
};
use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
    task::JoinHandle,
};

pub type DiscordMessage = serenity::all::Message;

#[derive(Debug)]
pub enum DiscordCommEvent {
    // GUI -> Discord
    Login(String),
    Logout,
    MessageSend(u64, String),
    // Discord -> GUI
    Ready,
    Error(String),
    MessageReceived(DiscordMessage),
}

pub struct DiscordManager {
    tx: Sender<DiscordCommEvent>,
    http_mutex: Arc<Mutex<Option<Arc<Http>>>>,
    client_thread: Option<JoinHandle<()>>,
    shard_manager: Option<Arc<ShardManager>>,
}

impl DiscordManager {
    pub fn new(tx: Sender<DiscordCommEvent>) -> Self {
        Self {
            tx: tx,
            http_mutex: Arc::new(Mutex::new(None)),
            client_thread: None,
            shard_manager: None,
        }
    }

    async fn send_to_gui(&self, event: DiscordCommEvent) {
        Self::tx_send(&self.tx, event).await;
    }

    async fn tx_send(tx: &Sender<DiscordCommEvent>, event: DiscordCommEvent) {
        tx.send(event).await.unwrap_or_else(|err| {
            eprintln!("Failed to send DiscordManager -> App: {:?}", err);
        });
    }

    async fn start_client(&mut self, token: String) {
        self.abort().await;

        let mut new_client = Self::new_client(token, self.tx.clone()).await;
        let tx = self.tx.clone();

        let http_mutex = self.http_mutex.clone();
        let http_mutex2 = self.http_mutex.clone();

        let mut http = http_mutex.lock().await;
        *http = Some(new_client.http.clone());

        self.shard_manager = Some(new_client.shard_manager.clone());

        let thread: JoinHandle<()> = tokio::spawn(async move {
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
                Self::tx_send(&tx, event).await;
            }
        });

        self.client_thread = Some(thread);
    }

    async fn unset_http(&mut self) {
        let mut http = self.http_mutex.lock().await;
        *http = None;
    }

    async fn abort(&mut self) {
        self.unset_http().await;

        if let Some(client_thread) = &self.client_thread {
            client_thread.abort();
        }

        if let Some(shard_manager) = &self.shard_manager {
            shard_manager.shutdown_all().await;
        }
    }

    pub async fn start(&mut self, mut rx: Receiver<DiscordCommEvent>) {
        // Important: http_mutex must not be locked and kept here, or other functions that use it will freeze

        loop {
            match rx.recv().await {
                Some(event) => match event {
                    DiscordCommEvent::Logout => self.abort().await,
                    DiscordCommEvent::Login(token) => {
                        self.start_client(token).await;
                    }
                    DiscordCommEvent::MessageSend(id, content) => {
                        let http = self.http_mutex.lock().await;

                        if let Some(http) = &http.to_owned() {
                            let msg_res = ChannelId::new(id).say(http, content).await;

                            if let Err(e) = msg_res {
                                self.send_to_gui(DiscordCommEvent::Error(format!(
                                    "Unable to send message: {}",
                                    e.to_string()
                                )))
                                .await;
                            }
                        } else {
                            self.send_to_gui(DiscordCommEvent::Error("Not logged in".to_string()))
                                .await;
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
    async fn send_to_gui(&self, event: DiscordCommEvent) {
        let tx = &self.tx;

        tx.send(event).await.unwrap_or_else(|err| {
            eprintln!("Failed to send DiscordHandler -> App: {:?}", err);
        });
    }
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, _ctx: Context, msg: Message) {
        println!("Received {}", &msg.content);

        self.send_to_gui(DiscordCommEvent::MessageReceived(msg))
            .await;
    }

    async fn ready(&self, _ctx: Context, _ready: Ready) {
        println!("Discord ready")
    }

    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {
        println!("Discord cache ready");
        self.send_to_gui(DiscordCommEvent::Ready).await;
    }
}
