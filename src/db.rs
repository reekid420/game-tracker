//! Database queries via SQLx.
//!
//! Handles CRUD for `games` and `play_sessions`. Uses migrations in `migrations/`.

use sqlx::{Row, SqlitePool};

use crate::models::Game;

pub async fn get_all_games(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games ORDER BY added_date DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_game_by_id(pool: &SqlitePool, id: i32) -> Result<Game, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

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

pub async fn insert_game(pool: &SqlitePool, game: &Game) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO games (title, platform, status, description, genre, release_year, \
         icon_path, cover_url, rawg_id, exe_path) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

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

pub async fn delete_game(pool: &SqlitePool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM games WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

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

pub async fn count_games(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM games")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

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

pub async fn total_playtime(pool: &SqlitePool) -> Result<f64, sqlx::Error> {
    let row = sqlx::query("SELECT COALESCE(SUM(playtime_hours), 0.0) as total FROM games")
        .fetch_one(pool)
        .await?;
    Ok(row.get("total"))
}
