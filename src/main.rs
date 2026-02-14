//! Game Tracker — multi-platform game library.
//!
//! Tracks games across PC, Switch, PS4, and emulators with play sessions,
//! ratings, and metadata. Uses Axum, SQLx/SQLite, HTMX, and RAWG API.

use axum::routing::{delete, get, post};
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tower_http::services::ServeDir;

mod db;
mod handlers;
mod icon_extract;
mod models;
mod rawg;

use handlers::AppState;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Database setup
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:game_tracker.db".to_string());
    let pool = SqlitePoolOptions::new()
        .connect(&format!("{}?mode=rwc", database_url))
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // RAWG API client
    let rawg_api_key =
        std::env::var("RAWG_API_KEY").expect("RAWG_API_KEY must be set in .env");
    let rawg_client = Arc::new(rawg::RawgClient::new(rawg_api_key));

    // Create static directories
    std::fs::create_dir_all("static/icons").ok();

    // Shared state
    let state = AppState {
        pool,
        rawg_client,
    };

    // Build router
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/add-form", get(handlers::add_game_form))
        .route("/search-rawg", post(handlers::search_rawg))
        .route("/games", post(handlers::create_game))
        .route("/games/{id}/status", post(handlers::update_status))
        .route("/games/{id}", delete(handlers::delete_game_handler))
        .route("/search", get(handlers::search))
        .route("/filter", get(handlers::filter_by_status))
        .route("/stats", get(handlers::stats))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    // Run server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::info!("Server running at http://127.0.0.1:3000");
    println!("🚀 Server running at http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
