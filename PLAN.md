```markdown
# Multi-Platform Game Tracker - Implementation Plan

## Architecture Overview

**Stack:**
- Backend: Axum 0.8 + SQLx + SQLite
- Frontend: HTMX + Askama templates
- External API: RAWG Video Games Database (free, 500k+ games)
- Icon extraction: `exeico` crate for Windows .exe files

**2026 Best Practices Baseline (applies to all phases):**
- Validate and sanitize all user inputs (title/platform/status/search)
- Handle errors explicitly; log failures without leaking secrets
- Secure config: keep `.env` out of git, avoid logging API keys, bind to `127.0.0.1` by default
- Dependency hygiene: keep `Cargo.lock` committed; run vulnerability scans (e.g., `cargo audit`) before releases
- Add minimal automated tests for DB queries and RAWG client; keep manual checklist
- If exposed beyond localhost, add auth and CSRF protection

---

## Phase 1: Project Setup (Day 1)

### 1.3 Project Structure
```
game-tracker/
├── src/
│   ├── main.rs
│   ├── db.rs           # Database queries
│   ├── models.rs       # Data structures
│   ├── handlers.rs     # Route handlers
│   ├── rawg.rs         # RAWG API client
│   └── icon_extract.rs # .exe icon extraction
├── templates/
│   ├── base.html
│   ├── index.html
│   ├── game_list.html
│   └── game_form.html
├── static/
│   ├── css/style.css
│   ├── js/htmx.min.js
│   └── icons/         # Stored game icons
├── migrations/
└── .env
```

---

## Phase 2: Database Schema (Day 1)

### 2.1 Create Migration
```bash
sqlx migrate add initial_schema
```

### 2.2 SQL Schema (migrations/001_initial_schema.sql)
```sql
CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    platform TEXT NOT NULL, -- PC, Switch, PS4, Emulator
    status TEXT DEFAULT 'Backlog', -- Playing, Completed, Backlog, Wishlist
    description TEXT,
    genre TEXT,
    release_year INTEGER,
    
    -- Image/Icon
    icon_path TEXT, -- Local path to stored icon
    cover_url TEXT, -- RAWG cover image URL
    
    -- Source tracking
    rawg_id INTEGER, -- RAWG game ID if found
    exe_path TEXT,   -- Path to .exe if PC game
    
    -- Stats
    playtime_hours REAL DEFAULT 0,
    rating INTEGER, -- Personal 1-10 rating
    
    -- Metadata
    added_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_played DATETIME,
    
    UNIQUE(title, platform)
);

CREATE TABLE play_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    session_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    duration_minutes INTEGER NOT NULL,
    notes TEXT,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE INDEX idx_games_status ON games(status);
CREATE INDEX idx_games_platform ON games(platform);
CREATE INDEX idx_sessions_game ON play_sessions(game_id);
```

### 2.3 Run Migration
```bash
sqlx migrate run --database-url sqlite:game_tracker.db
```

---

## Phase 3: RAWG API Integration (Day 2)

### 3.1 Get API Key
- Sign up at https://rawg.io/apidocs
- Free tier: 20,000 requests/month
- Add to `.env`: `RAWG_API_KEY=your_key_here`

### 3.2 RAWG Client (src/rawg.rs)
```rust
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Deserialize)]
pub struct RawgSearchResponse {
    pub results: Vec<RawgGame>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawgGame {
    pub id: i32,
    pub name: String,
    pub background_image: Option<String>,
    pub released: Option<String>,
    pub genres: Vec<Genre>,
    pub description_raw: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Genre {
    pub name: String,
}

pub struct RawgClient {
    client: Client,
    api_key: String,
}

impl RawgClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
    
    pub async fn search_game(&self, query: &str) -> Result<Vec<RawgGame>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.rawg.io/api/games?key={}&search={}&page_size=5",
            self.api_key, 
            urlencoding::encode(query)
        );
        
        let response: RawgSearchResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
            
        Ok(response.results)
    }
    
    pub async fn get_game_details(&self, game_id: i32) -> Result<RawgGame, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.rawg.io/api/games/{}?key={}",
            game_id, self.api_key
        );
        
        let game: RawgGame = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
            
        Ok(game)
    }
}
```

---

## Phase 4: Icon Extraction (Day 2)

### 4.1 Icon Extractor (src/icon_extract.rs)
```rust
use std::path::Path;
use std::fs;

pub fn extract_exe_icon(exe_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Only works on Windows
    #[cfg(target_os = "windows")]
    {
        let icon_data = exeico::get_exe_ico(exe_path)?;
        fs::write(output_path, icon_data)?;
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Icon extraction only supported on Windows".into())
    }
}

pub async fn download_icon(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    fs::write(output_path, bytes)?;
    Ok(())
}
```

---

## Phase 5: Core Backend (Days 3-4)

### 5.1 Models (src/models.rs)
```rust
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
```

### 5.2 Database Operations (src/db.rs)
```rust
use sqlx::{SqlitePool, Row};
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

pub async fn get_games_by_status(pool: &SqlitePool, status: &str) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games WHERE status = ? ORDER BY last_played DESC")
        .bind(status)
        .fetch_all(pool)
        .await
}

pub async fn insert_game(pool: &SqlitePool, game: &Game) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO games (title, platform, status, description, genre, release_year, 
         icon_path, cover_url, rawg_id, exe_path) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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

pub async fn update_game_status(pool: &SqlitePool, id: i32, status: &str) -> Result<(), sqlx::Error> {
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
    sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE title LIKE ? OR genre LIKE ? ORDER BY title"
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
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
    let rows = sqlx::query("SELECT platform, COUNT(*) as count FROM games GROUP BY platform")
        .fetch_all(pool)
        .await?;
    
    Ok(rows.iter().map(|row| {
        (row.get("platform"), row.get("count"))
    }).collect())
}

pub async fn count_by_status(pool: &SqlitePool) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = sqlx::query("SELECT status, COUNT(*) as count FROM games GROUP BY status")
        .fetch_all(pool)
        .await?;
    
    Ok(rows.iter().map(|row| {
        (row.get("status"), row.get("count"))
    }).collect())
}

pub async fn total_playtime(pool: &SqlitePool) -> Result<f64, sqlx::Error> {
    let row = sqlx::query("SELECT SUM(playtime_hours) as total FROM games")
        .fetch_one(pool)
        .await?;
    Ok(row.get("total"))
}
```

---

## Phase 6: Handlers (Days 4-5)

### 6.1 Main Handlers (src/handlers.rs)
```rust
use axum::{
    extract::{State, Path, Query},
    response::{Html, IntoResponse},
    Form,
};
use askama::Template;
use std::sync::Arc;
use sqlx::SqlitePool;
use crate::{db, models::*, rawg::RawgClient, icon_extract};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    games: Vec<Game>,
}

pub async fn index(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let games = db::get_all_games(&pool).await.unwrap_or_default();
    IndexTemplate { games }
}

#[derive(Template)]
#[template(path = "game_form.html")]
struct GameFormTemplate {
    rawg_results: Vec<crate::rawg::RawgGame>,
}

pub async fn add_game_form() -> impl IntoResponse {
    GameFormTemplate { 
        rawg_results: vec![] 
    }
}

pub async fn search_rawg(
    State(rawg_client): State<Arc<RawgClient>>,
    Form(form): Form<AddGameForm>,
) -> impl IntoResponse {
    let rawg_results = rawg_client
        .search_game(&form.title)
        .await
        .unwrap_or_default();
    
    GameFormTemplate { rawg_results }
}

#[derive(Template)]
#[template(path = "game_list.html")]
struct GameListTemplate {
    games: Vec<Game>,
}

pub async fn create_game(
    State(pool): State<SqlitePool>,
    State(rawg_client): State<Arc<RawgClient>>,
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
    
    // If user selected RAWG match
    if let Some(rawg_id) = form.rawg_id {
        if let Ok(rg) = rawg_client.get_game_details(rawg_id).await {
            game.description = rg.description_raw;
            game.genre = rg.genres.first().map(|g| g.name.clone());
            game.cover_url = rg.background_image.clone();
            game.rawg_id = Some(rawg_id);
            
            // Download icon from RAWG
            if let Some(img_url) = rg.background_image {
                let icon_path = format!("static/icons/{}.jpg", rawg_id);
                let _ = icon_extract::download_icon(&img_url, &icon_path).await;
                game.icon_path = Some(icon_path);
            }
        }
    }
    
    // If no RAWG match and exe_path provided, extract .exe icon
    if game.icon_path.is_none() && form.exe_path.is_some() {
        let exe_path = form.exe_path.unwrap();
        let icon_path = format!("static/icons/{}.ico", game.title.replace(" ", "_"));
        
        if icon_extract::extract_exe_icon(&exe_path, &icon_path).is_ok() {
            game.icon_path = Some(icon_path);
            game.exe_path = Some(exe_path);
        }
    }
    
    let _ = db::insert_game(&pool, &game).await;
    
    // Return updated game list (HTMX will swap this in)
    let games = db::get_all_games(&pool).await.unwrap_or_default();
    GameListTemplate { games }
}

#[derive(Template)]
#[template(path = "game_row.html")]
struct GameRowTemplate {
    game: Game,
}

pub async fn update_status(
    State(pool): State<SqlitePool>,
    Path(id): Path<i32>,
    Form(form): Form<StatusUpdate>,
) -> impl IntoResponse {
    let _ = db::update_game_status(&pool, id, &form.status).await;
    
    // Return single updated game row
    let game = db::get_game_by_id(&pool, id).await.unwrap();
    GameRowTemplate { game }
}

pub async fn delete_game_handler(
    State(pool): State<SqlitePool>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _ = db::delete_game(&pool, id).await;
    Html("") // Return empty (HTMX will remove element)
}

pub async fn search(
    State(pool): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let games = db::search_games(&pool, &params.q).await.unwrap_or_default();
    GameListTemplate { games }
}

pub async fn filter_by_status(
    State(pool): State<SqlitePool>,
    Query(params): Query<StatusUpdate>,
) -> impl IntoResponse {
    let games = if params.status.is_empty() {
        db::get_all_games(&pool).await.unwrap_or_default()
    } else {
        db::get_games_by_status(&pool, &params.status).await.unwrap_or_default()
    };
    GameListTemplate { games }
}

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate {
    total_games: i64,
    by_platform: Vec<(String, i64)>,
    by_status: Vec<(String, i64)>,
    total_playtime: f64,
}

pub async fn stats(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let total_games = db::count_games(&pool).await.unwrap_or(0);
    let by_platform = db::count_by_platform(&pool).await.unwrap_or_default();
    let by_status = db::count_by_status(&pool).await.unwrap_or_default();
    let total_playtime = db::total_playtime(&pool).await.unwrap_or(0.0);
    
    StatsTemplate {
        total_games,
        by_platform,
        by_status,
        total_playtime,
    }
}
```

---

## Phase 7: HTMX Frontend (Days 5-6)

### 7.1 Base Template (templates/base.html)
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Game Tracker</title>
    <script src="/static/js/htmx.min.js"></script>
    <link rel="stylesheet" href="/static/css/style.css">
</head>
<body>
    <header>
        <h1>🎮 Game Library Manager</h1>
        <nav>
            <a href="/">All Games</a>
            <a href="/stats">Stats</a>
        </nav>
    </header>
    <main>
        {% block content %}{% endblock %}
    </main>
</body>
</html>
```

### 7.2 Index Template (templates/index.html)
```html
{% extends "base.html" %}

{% block content %}
<div class="controls">
    <input 
        type="search" 
        name="q" 
        placeholder="Search games..."
        hx-get="/search" 
        hx-trigger="keyup changed delay:300ms"
        hx-target="#game-list"
    >
    
    <button 
        hx-get="/add-form" 
        hx-target="#add-game-modal"
        hx-swap="innerHTML">
        + Add Game
    </button>
    
    <select 
        hx-get="/filter" 
        hx-target="#game-list"
        name="status"
        hx-trigger="change">
        <option value="">All</option>
        <option value="Playing">Playing</option>
        <option value="Completed">Completed</option>
        <option value="Backlog">Backlog</option>
        <option value="Wishlist">Wishlist</option>
    </select>
</div>

<div id="add-game-modal"></div>

<div id="game-list">
    {% include "game_list.html" %}
</div>
{% endblock %}
```

### 7.3 Game List (templates/game_list.html)
```html
<div class="game-grid">
{% for game in games %}
    <div class="game-card" id="game-{{ game.id }}">
        {% if game.icon_path %}
            <img src="/{{ game.icon_path }}" alt="{{ game.title }}">
        {% else %}
            <div class="no-image">🎮</div>
        {% endif %}
        
        <h3>{{ game.title }}</h3>
        <span class="platform">{{ game.platform }}</span>
        
        <select 
            hx-post="/games/{{ game.id }}/status"
            hx-target="#game-{{ game.id }}"
            hx-swap="outerHTML"
            name="status">
            <option {% if game.status == "Backlog" %}selected{% endif %}>Backlog</option>
            <option {% if game.status == "Playing" %}selected{% endif %}>Playing</option>
            <option {% if game.status == "Completed" %}selected{% endif %}>Completed</option>
            <option {% if game.status == "Wishlist" %}selected{% endif %}>Wishlist</option>
        </select>
        
        {% if game.description %}
            <p class="description">{{ game.description }}</p>
        {% endif %}
        
        <p class="playtime">⏱️ {{ game.playtime_hours }}h</p>
        
        <button 
            class="delete-btn"
            hx-delete="/games/{{ game.id }}"
            hx-target="#game-{{ game.id }}"
            hx-swap="outerHTML swap:0.5s"
            hx-confirm="Delete {{ game.title }}?">
            🗑️
        </button>
    </div>
{% endfor %}
</div>
```

### 7.4 Game Row Template (templates/game_row.html)
```html
<div class="game-card" id="game-{{ game.id }}">
    {% if game.icon_path %}
        <img src="/{{ game.icon_path }}" alt="{{ game.title }}">
    {% else %}
        <div class="no-image">🎮</div>
    {% endif %}
    
    <h3>{{ game.title }}</h3>
    <span class="platform">{{ game.platform }}</span>
    
    <select 
        hx-post="/games/{{ game.id }}/status"
        hx-target="#game-{{ game.id }}"
        hx-swap="outerHTML"
        name="status">
        <option {% if game.status == "Backlog" %}selected{% endif %}>Backlog</option>
        <option {% if game.status == "Playing" %}selected{% endif %}>Playing</option>
        <option {% if game.status == "Completed" %}selected{% endif %}>Completed</option>
        <option {% if game.status == "Wishlist" %}selected{% endif %}>Wishlist</option>
    </select>
    
    {% if game.description %}
        <p class="description">{{ game.description }}</p>
    {% endif %}
    
    <p class="playtime">⏱️ {{ game.playtime_hours }}h</p>
    
    <button 
        class="delete-btn"
        hx-delete="/games/{{ game.id }}"
        hx-target="#game-{{ game.id }}"
        hx-swap="outerHTML swap:0.5s"
        hx-confirm="Delete {{ game.title }}?">
        🗑️
    </button>
</div>
```

### 7.5 Add Game Form (templates/game_form.html)
```html
<div class="modal">
    <div class="modal-content">
        <h2>Add New Game</h2>
        
        <form id="add-game-form">
            <input type="text" name="title" placeholder="Game Title" required id="game-title">
            
            <select name="platform" required>
                <option value="PC">PC</option>
                <option value="Switch">Switch</option>
                <option value="PS4">PS4</option>
                <option value="Emulator">Emulator</option>
            </select>
            
            <select name="status">
                <option value="Backlog">Backlog</option>
                <option value="Playing">Playing</option>
                <option value="Completed">Completed</option>
                <option value="Wishlist">Wishlist</option>
            </select>
            
            <input type="text" name="exe_path" placeholder="Path to .exe (PC games only)">
            
            <button 
                type="button" 
                hx-post="/search-rawg" 
                hx-include="[name='title']" 
                hx-target="#rawg-results">
                🔍 Search RAWG Database
            </button>
        </form>
        
        <div id="rawg-results">
            {% if rawg_results|length > 0%}
                <h3>Select Match:</h3>
                <div class="rawg-results-grid">
                {% for result in rawg_results %}
                    <label class="rawg-result">
                        <input type="radio" name="rawg_id" value="{{ result.id }}" form="add-game-form">
                        {% if result.background_image %}
                            <img src="{{ result.background_image }}" width="100">
                        {% endif %}
                        <div>
                            <strong>{{ result.name }}</strong><br>
                            <small>{{ result.released }}</small>
                        </div>
                    </label>
                {% endfor %}
                </div>
            {% endif %}
        </div>
        
        <div class="modal-actions">
            <button 
                hx-post="/games" 
                hx-include="#add-game-form, [name='rawg_id']:checked"
                hx-target="#game-list" 
                hx-swap="innerHTML">
                Add Game
            </button>
            <button type="button" onclick="document.getElementById('add-game-modal').innerHTML = ''">
                Cancel
            </button>
        </div>
    </div>
</div>
```

### 7.6 Stats Template (templates/stats.html)
```html
{% extends "base.html" %}

{% block content %}
<div class="stats-container">
    <h2>📊 Library Statistics</h2>
    
    <div class="stats-grid">
        <div class="stat-card">
            <h2>{{ total_games }}</h2>
            <p>Total Games</p>
        </div>
        
        <div class="stat-card">
            <h2>{{ total_playtime }}h</h2>
            <p>Total Playtime</p>
        </div>
        
        <div class="stat-card">
            <h3>By Platform</h3>
            {% for platform, count in by_platform %}
                <p>{{ platform }}: <strong>{{ count }}</strong></p>
            {% endfor %}
        </div>
        
        <div class="stat-card">
            <h3>By Status</h3>
            {% for status, count in by_status %}
                <p>{{ status }}: <strong>{{ count }}</strong></p>
            {% endfor %}
        </div>
    </div>
</div>
{% endblock %}
```

---

## Phase 8: Main Server Setup (Day 5)

### 8.1 Main Application (src/main.rs)
```rust
use axum::{
    routing::{get, post, delete},
    Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::services::ServeDir;
use std::env;

mod db;
mod models;
mod handlers;
mod rawg;
mod icon_extract;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Database setup
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:game_tracker.db".to_string());
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    // RAWG API client
    let rawg_api_key = env::var("RAWG_API_KEY")
        .expect("RAWG_API_KEY must be set in .env");
    let rawg_client = Arc::new(rawg::RawgClient::new(rawg_api_key));
    
    // Create static directories
    std::fs::create_dir_all("static/icons").ok();
    
    // Build router
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/add-form", get(handlers::add_game_form))
        .route("/search-rawg", post(handlers::search_rawg))
        .route("/games", post(handlers::create_game))
        .route("/games/:id/status", post(handlers::update_status))
        .route("/games/:id", delete(handlers::delete_game_handler))
        .route("/search", get(handlers::search))
        .route("/filter", get(handlers::filter_by_status))
        .route("/stats", get(handlers::stats))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool.clone())
        .with_state(rawg_client);
    
    // Run server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("🚀 Server running at http://127.0.0.1:3000");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
```

---

## Phase 9: Styling & Polish (Day 7)

### 9.1 CSS (static/css/style.css)
```css
:root {
    --bg: #1a1a1a;
    --card-bg: #2a2a2a;
    --text: #e0e0e0;
    --text-dim: #999;
    --accent: #00d9ff;
    --accent-hover: #00b8d4;
    --danger: #ff4444;
    --border: #444;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    background: var(--bg);
    color: var(--text);
    font-family: 'Segoe UI', -apple-system, system-ui, sans-serif;
    line-height: 1.6;
}

header {
    background: var(--card-bg);
    padding: 1.5rem 2rem;
    border-bottom: 2px solid var(--border);
    margin-bottom: 2rem;
}

header h1 {
    font-size: 1.8rem;
    margin-bottom: 0.5rem;
}

nav {
    display: flex;
    gap: 1rem;
}

nav a {
    color: var(--accent);
    text-decoration: none;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    transition: background 0.2s;
}

nav a:hover {
    background: rgba(0, 217, 255, 0.1);
}

main {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 2rem 2rem;
}

.controls {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
    flex-wrap: wrap;
}

input[type="search"],
select {
    padding: 0.75rem;
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text);
    font-size: 1rem;
}

input[type="search"] {
    flex: 1;
    min-width: 200px;
}

input[type="search"]:focus,
select:focus {
    outline: none;
    border-color: var(--accent);
}

button {
    background: var(--accent);
    color: #000;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 600;
    font-size: 1rem;
    transition: all 0.2s;
}

button:hover {
    background: var(--accent-hover);
    transform: translateY(-1px);
}

button:active {
    transform: translateY(0);
}

.game-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1.5rem;
}

.game-card {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 1rem;
    transition: transform 0.2s, box-shadow 0.2s;
    position: relative;
    display: flex;
    flex-direction: column;
}

.game-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.game-card img,
.no-image {
    width: 100%;
    height: 160px;
    object-fit: cover;
    border-radius: 8px;
    margin-bottom: 1rem;
}

.no-image {
    background: linear-gradient(135deg, #2a2a2a 0%, #1a1a1a 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 3rem;
}

.game-card h3 {
    font-size: 1.2rem;
    margin-bottom: 0.5rem;
    color: var(--text);
}

.platform {
    display: inline-block;
    background: var(--accent);
    color: #000;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-size: 0.85rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
}

.game-card select {
    margin: 0.75rem 0;
    width: 100%;
}

.description {
    font-size: 0.9rem;
    color: var(--text-dim);
    margin: 0.75rem 0;
    flex-grow: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
}

.playtime {
    color: var(--text-dim);
    font-size: 0.9rem;
    margin-bottom: 0.75rem;
}

.delete-btn {
    background: var(--danger);
    padding: 0.5rem;
    font-size: 0.9rem;
}

.delete-btn:hover {
    background: #ff6666;
}

/* Modal styles */
.modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
}

.modal-content {
    background: var(--card-bg);
    padding: 2rem;
    border-radius: 12px;
    max-width: 600px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
}

.modal h2 {
    margin-bottom: 1.5rem;
}

.modal input,
.modal select {
    width: 100%;
    margin-bottom: 1rem;
}

.modal-actions {
    display: flex;
    gap: 1rem;
    margin-top: 1.5rem;
}

.modal-actions button {
    flex: 1;
}

.modal-actions button:last-child {
    background: var(--border);
    color: var(--text);
}

.rawg-results-grid {
    display: grid;
    gap: 1rem;
    margin: 1rem 0;
}

.rawg-result {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 1rem;
    background: var(--bg);
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
}

.rawg-result:hover {
    background: #252525;
}

.rawg-result img {
    width: 100px;
    height: 60px;
    object-fit: cover;
    border-radius: 4px;
}

.rawg-result input[type="radio"] {
    margin: 0;
}

/* Stats page */
.stats-container h2 {
    margin-bottom: 2rem;
    font-size: 2rem;
}

.stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
}

.stat-card {
    background: var(--card-bg);
    padding: 2rem;
    border-radius: 12px;
    text-align: center;
}

.stat-card h2 {
    font-size: 3rem;
    color: var(--accent);
    margin-bottom: 0.5rem;
}

.stat-card h3 {
    margin-bottom: 1rem;
    color: var(--accent);
}

.stat-card p {
    margin: 0.5rem 0;
    font-size: 1.1rem;
}

/* Responsive design */
@media (max-width: 768px) {
    .game-grid {
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    }
    
    .controls {
        flex-direction: column;
    }
    
    input[type="search"] {
        width: 100%;
    }
}
```

---

## Phase 10: Download HTMX (Day 7)

Download HTMX from https://unpkg.com/htmx.org@2.0.0/dist/htmx.min.js and save to `static/js/htmx.min.js`

Or use CDN in base.html:
```html
<script src="https://unpkg.com/htmx.org@2.0.0"></script>
```

---

## Environment Setup

Create `.env` file:
```
DATABASE_URL=sqlite:game_tracker.db
RAWG_API_KEY=your_api_key_here
```

---

## Running the Project

```bash
# Run migrations
sqlx migrate run

# Start server
cargo run

# Access at http://127.0.0.1:3000
```

---

## Key Features Summary

✅ Manual game entry with title/platform  
✅ RAWG API integration - searches 500k+ game database  
✅ Auto-fetch description, cover art, genre from RAWG  
✅ Fallback to .exe icon extraction for PC games  
✅ HTMX dynamic UI - instant updates without page reloads  
✅ Status tracking with dropdown (Playing/Completed/Backlog/Wishlist)  
✅ Search/filter by title, genre, platform, status  
✅ Stats dashboard with game counts and playtime  
✅ SQLite database with proper relationships  

---

## Testing Checklist

- [ ] Add PC game with RAWG match → cover downloads
- [ ] Add PC game without RAWG match → .exe icon extracts
- [ ] Add Switch/PS4 game with RAWG match
- [ ] Update game status → updates instantly
- [ ] Search games → live filtering works
- [ ] Delete game → removes from list
- [ ] View stats → accurate counts
- [ ] Multi-platform filtering works

---

## Future Enhancements (Optional)

- [ ] Playtime logging with timer
- [ ] Import from Steam library CSV
- [ ] Game recommendations based on library
- [ ] Backup/export to JSON
- [ ] Dark/light theme toggle
- [ ] Custom tags and collections
```