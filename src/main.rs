#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod media_server;
mod models;
mod scanner;
mod streamer;
mod subtitles;

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
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(hwnd) = window.hwnd() {
                        unsafe {
                            #[link(name = "dwmapi")]
                            unsafe extern "system" {
                                fn DwmSetWindowAttribute(
                                    hwnd: *mut std::ffi::c_void,
                                    dwAttribute: u32,
                                    pvAttribute: *const std::ffi::c_void,
                                    cbAttribute: u32,
                                ) -> i32;
                            }

                            let dark_mode: i32 = 1;
                            // DWMWA_USE_IMMERSIVE_DARK_MODE = 20
                            DwmSetWindowAttribute(
                                hwnd.0 as *mut std::ffi::c_void,
                                20,
                                &dark_mode as *const _ as *const std::ffi::c_void,
                                std::mem::size_of::<i32>() as u32,
                            );

                            // DWMWA_CAPTION_COLOR = 35 (Windows 11 build 22000+)
                            // COLORREF: 0x00BBGGRR -> #131222 = R:0x13, G:0x12, B:0x22 -> 0x00221213
                            let caption_color: u32 = 0x00221213;
                            DwmSetWindowAttribute(
                                hwnd.0 as *mut std::ffi::c_void,
                                35,
                                &caption_color as *const _ as *const std::ffi::c_void,
                                std::mem::size_of::<u32>() as u32,
                            );

                            // DWMWA_TEXT_COLOR = 36 (White/light text)
                            let text_color: u32 = 0x00FAF4F5;
                            DwmSetWindowAttribute(
                                hwnd.0 as *mut std::ffi::c_void,
                                36,
                                &text_color as *const _ as *const std::ffi::c_void,
                                std::mem::size_of::<u32>() as u32,
                            );

                            // DWMWA_BORDER_COLOR = 34
                            let border_color: u32 = 0x0048282B;
                            DwmSetWindowAttribute(
                                hwnd.0 as *mut std::ffi::c_void,
                                34,
                                &border_color as *const _ as *const std::ffi::c_void,
                                std::mem::size_of::<u32>() as u32,
                            );
                        }
                    }
                }
            }
            Ok(())
        })
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
