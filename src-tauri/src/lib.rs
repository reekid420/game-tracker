//! Tauri application bootstrap and shared state initialization.
//!
//! This module wires the desktop runtime to the shared core crate by creating
//! a [`game_tracker_core::service::GameService`] instance and registering
//! command handlers exposed to the React frontend.

use std::path::PathBuf;
use std::sync::Arc;

use game_tracker_core::rawg::RawgClient;
use game_tracker_core::service::GameService;
use sqlx::sqlite::SqlitePoolOptions;
use tauri::Manager;
use tokio::sync::Mutex;

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Start the Tauri desktop runtime.
///
/// During setup, this function:
/// - resolves app data paths for SQLite and icon storage
/// - creates a SQLx pool and runs migrations
/// - initializes `GameService` state
/// - registers command handlers for frontend `invoke` calls
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize logging
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Resolve app data directory for DB and icons
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir.join("game_tracker.db");
            let icons_dir = app_data_dir.join("icons");

            let db_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

            // Load .env for RAWG key (optional in desktop — can fall back to empty)
            dotenv::dotenv().ok();
            let rawg_api_key = std::env::var("RAWG_API_KEY").unwrap_or_default();

            let handle = app.handle().clone();

            // Spawn async setup on the Tokio runtime
            tauri::async_runtime::spawn(async move {
                let pool = SqlitePoolOptions::new()
                    .connect(&db_url)
                    .await
                    .expect("Failed to connect to database");

                sqlx::migrate!("../migrations")
                    .run(&pool)
                    .await
                    .expect("Failed to run migrations");

                let rawg_client = Arc::new(RawgClient::new(rawg_api_key));
                let service = GameService::new(pool, rawg_client, icons_dir);
                let service_state: commands::ServiceState = Arc::new(Mutex::new(service));

                handle.manage(service_state);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::search_games,
            commands::filter_games,
            commands::create_game,
            commands::update_game_status,
            commands::delete_game,
            commands::get_game_stats,
            commands::search_rawg,
            commands::index_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
