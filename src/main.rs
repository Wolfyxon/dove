#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::process::exit;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    app::App,
    discord::{DiscordCommEvent, DiscordManager},
    utils::comm::{COMM_BUFFER_SIZE, MPSCChannel},
};

mod app;
mod config;
mod commands;
mod discord;
mod utils;
mod crypto;

#[tokio::main]
async fn main() {
    let (tx_dc_to_gui, rx_dc_to_gui): MPSCChannel<DiscordCommEvent> =
        mpsc::channel(COMM_BUFFER_SIZE);
    let (tx_gui_to_dc, rx_gui_to_dc): MPSCChannel<DiscordCommEvent> =
        mpsc::channel(COMM_BUFFER_SIZE);

    let _discord_thread = tokio::spawn(async {
        start_discord(tx_dc_to_gui, rx_gui_to_dc).await;
    });

    start_gui(tx_gui_to_dc, rx_dc_to_gui); // NOTE: egui must run on main thread
}

fn start_gui(tx_gui_to_dc: Sender<DiscordCommEvent>, rx_dc_to_gui: Receiver<DiscordCommEvent>) {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([400.0, 200.0])
        .with_transparent(true)
        .with_always_on_top()
        /*.with_resizable(false)*/;

    let options = eframe::NativeOptions {
        viewport: viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Dove",
        options,
        Box::new(|_creation_ctx| Ok(Box::new(App::new(tx_gui_to_dc, rx_dc_to_gui)))),
    )
    .unwrap_or_else(|err| {
        eprintln!("Start failed: {}", err);
        exit(1);
    });
}

async fn start_discord(
    tx_dc_to_gui: Sender<DiscordCommEvent>,
    rx_gui_to_dc: Receiver<DiscordCommEvent>,
) {
    let mut mgr = DiscordManager::new(tx_dc_to_gui);
    mgr.start(rx_gui_to_dc).await;
}
