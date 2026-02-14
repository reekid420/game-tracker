-- Game Tracker - Initial schema (Phase 2)
-- Matches PLAN.md Phase 2.2
--
-- games: title, platform, status (Playing/Completed/Backlog/Wishlist), metadata,
--        icon_path, cover_url, rawg_id, exe_path, playtime_hours, rating
-- play_sessions: game_id, session_date, duration_minutes, notes (FK CASCADE)

CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    platform TEXT NOT NULL,
    status TEXT DEFAULT 'Backlog',
    description TEXT,
    genre TEXT,
    release_year INTEGER,
    icon_path TEXT,
    cover_url TEXT,
    rawg_id INTEGER,
    exe_path TEXT,
    playtime_hours REAL DEFAULT 0,
    rating INTEGER,
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
