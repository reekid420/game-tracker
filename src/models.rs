//! Data structures for game tracker.
//!
//! Key types: `Game` (title, platform, status, playtime, rating, icon/cover),
//! form structs for Axum extractors, and query parameter types.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
}

#[derive(Debug, Deserialize)]
pub struct AddGameForm {
    pub title: String,
    pub platform: String,
    pub status: String,
    pub exe_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGameForm {
    pub title: String,
    pub platform: String,
    pub status: String,
    pub rawg_id: Option<i32>,
    pub exe_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdate {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
}
