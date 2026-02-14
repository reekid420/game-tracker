//! SQLx query helpers for game library persistence.
//!
//! This module intentionally keeps business logic minimal and focuses on
//! deterministic data access operations.

use sqlx::{Row, SqlitePool};

use crate::models::Game;

/// Fetch all games ordered by most recently added.
pub async fn get_all_games(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games ORDER BY added_date DESC")
        .fetch_all(pool)
        .await
}

/// Fetch a single game by primary key.
pub async fn get_game_by_id(pool: &SqlitePool, id: i32) -> Result<Game, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

/// Fetch games matching a status value, most recently played first.
pub async fn get_games_by_status(
    pool: &SqlitePool,
    status: &str,
) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE status = ? ORDER BY last_played DESC",
    )
    .bind(status)
    .fetch_all(pool)
    .await
}

/// Insert a game and return the newly assigned SQLite row id.
pub async fn insert_game(pool: &SqlitePool, game: &Game) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO games (title, platform, status, description, genre, release_year, \
         icon_path, cover_url, rawg_id, exe_path, source, source_id, install_path) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&game.title)
    .bind(&game.platform)
    .bind(&game.status)
    .bind(&game.description)
    .bind(&game.genre)
    .bind(&game.release_year)
    .bind(&game.icon_path)
    .bind(&game.cover_url)
    .bind(&game.rawg_id)
    .bind(&game.exe_path)
    .bind(&game.source)
    .bind(&game.source_id)
    .bind(&game.install_path)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Upsert by `(source, source_id)` and return the affected game id.
///
/// Existing rows are updated with latest title/install/executable path so
/// repeated index runs refresh metadata instead of creating duplicates.
pub async fn upsert_game_by_source(pool: &SqlitePool, game: &Game) -> Result<i64, sqlx::Error> {
    // Check if game already exists by source + source_id
    let existing = sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE source = ? AND source_id = ?",
    )
    .bind(&game.source)
    .bind(&game.source_id)
    .fetch_optional(pool)
    .await?;

    if let Some(existing) = existing {
        // Update install path and exe path if they changed
        sqlx::query(
            "UPDATE games SET install_path = ?, exe_path = ?, title = ? WHERE id = ?",
        )
        .bind(&game.install_path)
        .bind(&game.exe_path)
        .bind(&game.title)
        .bind(existing.id)
        .execute(pool)
        .await?;
        Ok(existing.id as i64)
    } else {
        insert_game(pool, game).await
    }
}

/// Update a game's status and stamp `last_played` with current time.
pub async fn update_game_status(
    pool: &SqlitePool,
    id: i32,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE games SET status = ?, last_played = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a game by primary key.
pub async fn delete_game(pool: &SqlitePool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM games WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Search games by title or genre using a `%LIKE%` pattern.
pub async fn search_games(pool: &SqlitePool, query: &str) -> Result<Vec<Game>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE title LIKE ? OR genre LIKE ? ORDER BY title",
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await
}

/// Count total games in the library.
pub async fn count_games(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM games")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

/// Return game counts grouped by platform.
pub async fn count_by_platform(pool: &SqlitePool) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT platform, COUNT(*) as count FROM games GROUP BY platform")
            .fetch_all(pool)
            .await?;

    Ok(rows
        .iter()
        .map(|row| (row.get("platform"), row.get("count")))
        .collect())
}

/// Return game counts grouped by status.
pub async fn count_by_status(pool: &SqlitePool) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT status, COUNT(*) as count FROM games GROUP BY status")
            .fetch_all(pool)
            .await?;

    Ok(rows
        .iter()
        .map(|row| (row.get("status"), row.get("count")))
        .collect())
}

/// Sum all stored playtime values in hours.
pub async fn total_playtime(pool: &SqlitePool) -> Result<f64, sqlx::Error> {
    let row = sqlx::query("SELECT COALESCE(SUM(playtime_hours), 0.0) as total FROM games")
        .fetch_one(pool)
        .await?;
    Ok(row.get("total"))
}
