//! Shared model and DTO definitions for the game tracker domain.
//!
//! Types in this module are used by:
//! - database query mapping (`sqlx::FromRow`)
//! - Tauri command input/output payloads
//! - launcher indexer handoff into the service layer

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Persisted game record stored in SQLite and returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Game {
    pub id: i32,
    pub title: String,
    pub platform: String,
    pub status: String,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub release_year: Option<i32>,
    pub icon_path: Option<String>,
    pub cover_url: Option<String>,
    pub rawg_id: Option<i32>,
    pub exe_path: Option<String>,
    pub playtime_hours: f32,
    pub rating: Option<i32>,
    pub added_date: String,
    pub last_played: Option<String>,
    /// Source launcher: "manual", "steam", "epic"
    pub source: Option<String>,
    /// External ID from launcher (Steam AppID, Epic AppName)
    pub source_id: Option<String>,
    /// Path where the game is installed
    pub install_path: Option<String>,
}

/// Input payload used when creating a new game entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameInput {
    pub title: String,
    pub platform: String,
    pub status: String,
    pub rawg_id: Option<i32>,
    pub exe_path: Option<String>,
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub install_path: Option<String>,
}

/// Input payload for updating only a game's status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdateInput {
    pub status: String,
}

/// Input payload for free-text library search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    pub q: String,
}

/// Game discovered from a launcher before it is upserted into SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredGame {
    pub title: String,
    pub platform: String,
    pub exe_path: Option<String>,
    pub install_path: Option<String>,
    pub source: String,
    pub source_id: String,
}

/// Aggregated library metrics shown in the stats view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    pub total_games: i64,
    pub by_platform: Vec<(String, i64)>,
    pub by_status: Vec<(String, i64)>,
    pub total_playtime: f64,
}
