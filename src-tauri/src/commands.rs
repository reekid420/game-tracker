//! Tauri commands — the frontend-backend API boundary.

use game_tracker_core::models::*;
use game_tracker_core::rawg::RawgGame;
use game_tracker_core::service::{GameService, IndexResult};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Shared mutable service state managed by Tauri.
pub type ServiceState = Arc<Mutex<GameService>>;

// ---- Games CRUD -------------------------------------------------------------

#[tauri::command]
/// Return all games in the library.
pub async fn list_games(service: State<'_, ServiceState>) -> Result<Vec<Game>, String> {
    let svc = service.lock().await;
    svc.list_games().await
}

#[tauri::command]
/// Search games by free-text query (title/genre).
pub async fn search_games(
    service: State<'_, ServiceState>,
    query: String,
) -> Result<Vec<Game>, String> {
    let svc = service.lock().await;
    svc.search_games(&query).await
}

#[tauri::command]
/// Filter games by status (`Playing`, `Completed`, etc.).
pub async fn filter_games(
    service: State<'_, ServiceState>,
    status: String,
) -> Result<Vec<Game>, String> {
    let svc = service.lock().await;
    svc.filter_games(&status).await
}

#[tauri::command]
/// Create a new game from user input and optional enrichment fields.
pub async fn create_game(
    service: State<'_, ServiceState>,
    input: CreateGameInput,
) -> Result<Game, String> {
    let svc = service.lock().await;
    svc.create_game(input).await
}

#[tauri::command]
/// Update the status for a game id.
pub async fn update_game_status(
    service: State<'_, ServiceState>,
    id: i32,
    status: String,
) -> Result<(), String> {
    let svc = service.lock().await;
    svc.update_game_status(id, &status).await
}

#[tauri::command]
/// Delete a game by id.
pub async fn delete_game(
    service: State<'_, ServiceState>,
    id: i32,
) -> Result<(), String> {
    let svc = service.lock().await;
    svc.delete_game(id).await
}

// ---- Stats ------------------------------------------------------------------

#[tauri::command]
/// Return aggregate library statistics.
pub async fn get_game_stats(
    service: State<'_, ServiceState>,
) -> Result<GameStats, String> {
    let svc = service.lock().await;
    svc.get_stats().await
}

// ---- RAWG -------------------------------------------------------------------

#[tauri::command]
/// Proxy RAWG search to support manual game creation.
pub async fn search_rawg(
    service: State<'_, ServiceState>,
    query: String,
) -> Result<Vec<RawgGame>, String> {
    let svc = service.lock().await;
    svc.search_rawg(&query).await
}

// ---- Indexing ----------------------------------------------------------------

#[tauri::command]
/// Run launcher indexing for all supported sources.
pub async fn index_now(
    service: State<'_, ServiceState>,
) -> Result<IndexResult, String> {
    let svc = service.lock().await;
    svc.index_all().await
}
