//! GameService — orchestrates business logic for game CRUD, indexing, and enrichment.

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::models::*;
use crate::{db, icon_extract, indexers, rawg::RawgClient};

/// High-level coordinator for library operations used by Tauri commands.
pub struct GameService {
    /// Shared SQLite connection pool.
    pub pool: SqlitePool,
    /// Client used for optional RAWG metadata enrichment.
    pub rawg_client: Arc<RawgClient>,
    /// Directory where downloaded covers and extracted icons are stored.
    pub icons_dir: PathBuf,
}

impl GameService {
    /// Create a new service and ensure icon storage exists.
    pub fn new(pool: SqlitePool, rawg_client: Arc<RawgClient>, icons_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&icons_dir).ok();
        Self {
            pool,
            rawg_client,
            icons_dir,
        }
    }

    // ---- CRUD ---------------------------------------------------------------

    /// Return the full game library.
    pub async fn list_games(&self) -> Result<Vec<Game>, String> {
        db::get_all_games(&self.pool).await.map_err(|e| e.to_string())
    }

    /// Search the library by title/genre.
    pub async fn search_games(&self, query: &str) -> Result<Vec<Game>, String> {
        db::search_games(&self.pool, query).await.map_err(|e| e.to_string())
    }

    /// Filter the library by status. Empty status returns all games.
    pub async fn filter_games(&self, status: &str) -> Result<Vec<Game>, String> {
        if status.is_empty() {
            db::get_all_games(&self.pool).await.map_err(|e| e.to_string())
        } else {
            db::get_games_by_status(&self.pool, status)
                .await
                .map_err(|e| e.to_string())
        }
    }

    /// Create a game and optionally enrich it from RAWG or executable icon data.
    pub async fn create_game(&self, input: CreateGameInput) -> Result<Game, String> {
        let mut game = Game {
            id: 0,
            title: input.title.clone(),
            platform: input.platform,
            status: input.status,
            description: None,
            genre: None,
            release_year: None,
            icon_path: None,
            cover_url: None,
            rawg_id: None,
            exe_path: None,
            playtime_hours: 0.0,
            rating: None,
            added_date: String::new(),
            last_played: None,
            source: input.source,
            source_id: input.source_id,
            install_path: input.install_path,
        };

        // Enrich from RAWG if a match was selected
        if let Some(rawg_id) = input.rawg_id {
            if let Ok(rg) = self.rawg_client.get_game_details(rawg_id).await {
                game.description = rg.description_raw;
                game.genre = rg.genres.first().map(|g| g.name.clone());
                game.cover_url = rg.background_image.clone();
                game.rawg_id = Some(rawg_id);

                if let Some(ref img_url) = rg.background_image {
                    let icon_file = self.icons_dir.join(format!("{}.jpg", rawg_id));
                    let icon_path_str = icon_file.to_string_lossy().to_string();
                    let _ = icon_extract::download_icon(img_url, &icon_path_str).await;
                    game.icon_path = Some(icon_path_str);
                }
            }
        }

        // Fallback: extract .exe icon for PC games
        if game.icon_path.is_none() {
            if let Some(ref exe_path) = input.exe_path {
                if !exe_path.is_empty() {
                    let icon_file = self
                        .icons_dir
                        .join(format!("{}.ico", game.title.replace(' ', "_")));
                    let icon_path_str = icon_file.to_string_lossy().to_string();
                    if icon_extract::extract_exe_icon(exe_path, &icon_path_str).is_ok() {
                        game.icon_path = Some(icon_path_str);
                        game.exe_path = Some(exe_path.clone());
                    }
                }
            }
        }

        let id = db::insert_game(&self.pool, &game)
            .await
            .map_err(|e| e.to_string())?;
        game.id = id as i32;
        Ok(game)
    }

    /// Update status for a game by id.
    pub async fn update_game_status(&self, id: i32, status: &str) -> Result<(), String> {
        db::update_game_status(&self.pool, id, status)
            .await
            .map_err(|e| e.to_string())
    }

    /// Delete a game by id.
    pub async fn delete_game(&self, id: i32) -> Result<(), String> {
        db::delete_game(&self.pool, id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Compute aggregate statistics for the stats view.
    pub async fn get_stats(&self) -> Result<GameStats, String> {
        let total_games = db::count_games(&self.pool).await.map_err(|e| e.to_string())?;
        let by_platform = db::count_by_platform(&self.pool).await.map_err(|e| e.to_string())?;
        let by_status = db::count_by_status(&self.pool).await.map_err(|e| e.to_string())?;
        let total_playtime = db::total_playtime(&self.pool).await.map_err(|e| e.to_string())?;

        Ok(GameStats {
            total_games,
            by_platform,
            by_status,
            total_playtime,
        })
    }

    // ---- RAWG ---------------------------------------------------------------

    /// Search RAWG from the service layer.
    pub async fn search_rawg(&self, query: &str) -> Result<Vec<crate::rawg::RawgGame>, String> {
        self.rawg_client
            .search_game(query)
            .await
            .map_err(|e| e.to_string())
    }

    // ---- Indexing ------------------------------------------------------------

    /// Run indexing for all supported launchers.
    ///
    /// `upserted` counts successful upsert operations (both inserts and updates).
    pub async fn index_all(&self) -> Result<IndexResult, String> {
        let mut total_discovered = 0u32;
        let mut total_new = 0u32;

        // Steam
        match indexers::steam::scan_steam_games() {
            Ok(steam_games) => {
                total_discovered += steam_games.len() as u32;
                for dg in steam_games {
                    let game = discovered_to_game(&dg);
                    match db::upsert_game_by_source(&self.pool, &game).await {
                        Ok(_id) => total_new += 1,
                        Err(e) => tracing::warn!("Failed to upsert Steam game {}: {}", dg.title, e),
                    }
                }
            }
            Err(e) => tracing::warn!("Steam indexing failed: {}", e),
        }

        // Epic
        match indexers::epic::scan_epic_games() {
            Ok(epic_games) => {
                total_discovered += epic_games.len() as u32;
                for dg in epic_games {
                    let game = discovered_to_game(&dg);
                    match db::upsert_game_by_source(&self.pool, &game).await {
                        Ok(_id) => total_new += 1,
                        Err(e) => tracing::warn!("Failed to upsert Epic game {}: {}", dg.title, e),
                    }
                }
            }
            Err(e) => tracing::warn!("Epic indexing failed: {}", e),
        }

        Ok(IndexResult {
            discovered: total_discovered,
            upserted: total_new,
        })
    }
}

/// Summary of an indexing pass.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexResult {
    /// Number of launcher entries discovered by indexers.
    pub discovered: u32,
    /// Number of rows successfully upserted into the database.
    pub upserted: u32,
}

fn discovered_to_game(dg: &DiscoveredGame) -> Game {
    Game {
        id: 0,
        title: dg.title.clone(),
        platform: dg.platform.clone(),
        status: "Backlog".to_string(),
        description: None,
        genre: None,
        release_year: None,
        icon_path: None,
        cover_url: None,
        rawg_id: None,
        exe_path: dg.exe_path.clone(),
        playtime_hours: 0.0,
        rating: None,
        added_date: String::new(),
        last_played: None,
        source: Some(dg.source.clone()),
        source_id: Some(dg.source_id.clone()),
        install_path: dg.install_path.clone(),
    }
}
