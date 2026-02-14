-- Add source/launcher tracking fields for auto-indexing (Steam, Epic)

ALTER TABLE games ADD COLUMN source TEXT;         -- "manual", "steam", "epic"
ALTER TABLE games ADD COLUMN source_id TEXT;       -- Steam AppID, Epic AppName
ALTER TABLE games ADD COLUMN install_path TEXT;    -- Full install directory path

CREATE INDEX idx_games_source ON games(source);
CREATE UNIQUE INDEX idx_games_source_id ON games(source, source_id);
