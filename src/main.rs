#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod media_server;
mod models;
mod scanner;
mod streamer;

use commands::*;
use db::AppDatabase;
use std::sync::Arc;
use streamer::start_streaming_server;
use tokio::sync::RwLock;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "offclass=info".into()),
        )
        .init();

    info!("Initializing OffClass Tauri Application...");

    // Initialize Database
    let database = AppDatabase::init();
    let shared_db = Arc::new(RwLock::new(database));

    // Start background video streaming server
    let streaming_port = start_streaming_server(shared_db.clone())
        .await
        .expect("Failed to start video streaming server");

    tauri::Builder::default()
        .manage(shared_db)
        .manage(StreamPort(streaming_port))
        .invoke_handler(tauri::generate_handler![
            get_streaming_port,
            list_courses,
            scan_course,
            rescan_course,
            get_course,
            delete_course,
            update_progress,
            update_position,
            update_duration,
            add_note,
            delete_note,
            get_settings,
            update_settings,
            pick_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
