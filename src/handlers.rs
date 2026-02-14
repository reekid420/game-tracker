//! Route handlers for Axum.
//!
//! Serves index, game list, game form (add/edit), stats, search, and filter.
//! Uses Askama templates and HTMX for partial updates.

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Form,
};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::{db, icon_extract, models::*, rawg::RawgClient};

/// Render an Askama template into an `Html<String>` response.
fn render<T: Template>(tmpl: T) -> Html<String> {
    Html(tmpl.render().expect("template rendering failed"))
}

// ---------------------------------------------------------------------------
// Shared application state (Axum 0.8 requires a single state type)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub rawg_client: Arc<RawgClient>,
}

// ---------------------------------------------------------------------------
// Askama template structs
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    games: Vec<Game>,
}

#[derive(Template)]
#[template(path = "game_form.html")]
struct GameFormTemplate {
    rawg_results: Vec<crate::rawg::RawgGame>,
}

#[derive(Template)]
#[template(path = "game_list.html")]
struct GameListTemplate {
    games: Vec<Game>,
}

#[derive(Template)]
#[template(path = "game_row.html")]
struct GameRowTemplate {
    game: Game,
}

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate {
    total_games: i64,
    by_platform: Vec<(String, i64)>,
    by_status: Vec<(String, i64)>,
    total_playtime: f64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET / — render the main index page with all games.
pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let games = db::get_all_games(&state.pool).await.unwrap_or_default();
    render(IndexTemplate { games })
}

/// GET /add-form — render the "Add Game" modal form (empty RAWG results).
pub async fn add_game_form() -> impl IntoResponse {
    render(GameFormTemplate {
        rawg_results: vec![],
    })
}

/// POST /search-rawg — search RAWG and re-render the form with results.
pub async fn search_rawg(
    State(state): State<AppState>,
    Form(form): Form<AddGameForm>,
) -> impl IntoResponse {
    let rawg_results = state
        .rawg_client
        .search_game(&form.title)
        .await
        .unwrap_or_default();

    render(GameFormTemplate { rawg_results })
}

/// POST /games — create a new game, optionally enriching from RAWG.
pub async fn create_game(
    State(state): State<AppState>,
    Form(form): Form<CreateGameForm>,
) -> impl IntoResponse {
    let mut game = Game {
        id: 0,
        title: form.title.clone(),
        platform: form.platform,
        status: form.status,
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
    };

    // Enrich from RAWG if the user selected a match
    if let Some(rawg_id) = form.rawg_id {
        if let Ok(rg) = state.rawg_client.get_game_details(rawg_id).await {
            game.description = rg.description_raw;
            game.genre = rg.genres.first().map(|g| g.name.clone());
            game.cover_url = rg.background_image.clone();
            game.rawg_id = Some(rawg_id);

            // Download cover image from RAWG
            if let Some(ref img_url) = rg.background_image {
                let icon_path = format!("static/icons/{}.jpg", rawg_id);
                let _ = icon_extract::download_icon(img_url, &icon_path).await;
                game.icon_path = Some(icon_path);
            }
        }
    }

    // Fallback: extract .exe icon for PC games without a RAWG cover
    if game.icon_path.is_none() {
        if let Some(ref exe_path) = form.exe_path {
            if !exe_path.is_empty() {
                let icon_path = format!(
                    "static/icons/{}.ico",
                    game.title.replace(' ', "_")
                );
                if icon_extract::extract_exe_icon(exe_path, &icon_path).is_ok() {
                    game.icon_path = Some(icon_path);
                    game.exe_path = Some(exe_path.clone());
                }
            }
        }
    }

    let _ = db::insert_game(&state.pool, &game).await;

    // Return updated full game list (HTMX swaps it into #game-list)
    let games = db::get_all_games(&state.pool).await.unwrap_or_default();
    render(GameListTemplate { games })
}

/// POST /games/:id/status — update a game's status and return the row.
pub async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<StatusUpdate>,
) -> impl IntoResponse {
    let _ = db::update_game_status(&state.pool, id, &form.status).await;
    match db::get_game_by_id(&state.pool, id).await {
        Ok(game) => render(GameRowTemplate { game }).into_response(),
        Err(_) => Html("".to_string()).into_response(),
    }
}

/// DELETE /games/:id — remove a game and return empty HTML.
pub async fn delete_game_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _ = db::delete_game(&state.pool, id).await;
    Html("".to_string()) // HTMX removes the element
}

/// GET /search?q=... — live search games by title/genre.
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let games = db::search_games(&state.pool, &params.q)
        .await
        .unwrap_or_default();
    render(GameListTemplate { games })
}

/// GET /filter?status=... — filter games by status.
pub async fn filter_by_status(
    State(state): State<AppState>,
    Query(params): Query<StatusUpdate>,
) -> impl IntoResponse {
    let games = if params.status.is_empty() {
        db::get_all_games(&state.pool).await.unwrap_or_default()
    } else {
        db::get_games_by_status(&state.pool, &params.status)
            .await
            .unwrap_or_default()
    };
    render(GameListTemplate { games })
}

/// GET /stats — library statistics page.
pub async fn stats(State(state): State<AppState>) -> impl IntoResponse {
    let total_games = db::count_games(&state.pool).await.unwrap_or(0);
    let by_platform = db::count_by_platform(&state.pool).await.unwrap_or_default();
    let by_status = db::count_by_status(&state.pool).await.unwrap_or_default();
    let total_playtime = db::total_playtime(&state.pool).await.unwrap_or(0.0);

    render(StatsTemplate {
        total_games,
        by_platform,
        by_status,
        total_playtime,
    })
}
