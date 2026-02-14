import { invoke } from "@tauri-apps/api/core";

// ---- Types (match Rust DTOs) ----

/** Persisted game row returned by the backend. */
export interface Game {
  id: number;
  title: string;
  platform: string;
  status: string;
  description: string | null;
  genre: string | null;
  release_year: number | null;
  icon_path: string | null;
  cover_url: string | null;
  rawg_id: number | null;
  exe_path: string | null;
  playtime_hours: number;
  rating: number | null;
  added_date: string;
  last_played: string | null;
  source: string | null;
  source_id: string | null;
  install_path: string | null;
}

/** Payload used to create a new game entry. */
export interface CreateGameInput {
  title: string;
  platform: string;
  status: string;
  rawg_id?: number | null;
  exe_path?: string | null;
  source?: string | null;
  source_id?: string | null;
  install_path?: string | null;
}

/** Minimal RAWG match shown in the add-game flow. */
export interface RawgGame {
  id: number;
  name: string;
  background_image: string | null;
  released: string | null;
  genres: { name: string }[];
  description_raw: string | null;
}

/** Aggregated library metrics for the stats screen. */
export interface GameStats {
  total_games: number;
  by_platform: [string, number][];
  by_status: [string, number][];
  total_playtime: number;
}

/** Result returned after running launcher indexing. */
export interface IndexResult {
  discovered: number;
  upserted: number;
}

// ---- API functions ----

/** Fetch all games currently stored in the library. */
export async function listGames(): Promise<Game[]> {
  return invoke<Game[]>("list_games");
}

/** Search games by title/genre query text. */
export async function searchGames(query: string): Promise<Game[]> {
  return invoke<Game[]>("search_games", { query });
}

/** Filter games by status value. */
export async function filterGames(status: string): Promise<Game[]> {
  return invoke<Game[]>("filter_games", { status });
}

/** Create a game using manual input and optional enrichment fields. */
export async function createGame(input: CreateGameInput): Promise<Game> {
  return invoke<Game>("create_game", { input });
}

/** Update the status of a single game. */
export async function updateGameStatus(
  id: number,
  status: string
): Promise<void> {
  return invoke("update_game_status", { id, status });
}

/** Delete a game by id. */
export async function deleteGame(id: number): Promise<void> {
  return invoke("delete_game", { id });
}

/** Load aggregate statistics for the stats dashboard. */
export async function getGameStats(): Promise<GameStats> {
  return invoke<GameStats>("get_game_stats");
}

/** Search RAWG for candidate metadata matches by title. */
export async function searchRawg(query: string): Promise<RawgGame[]> {
  return invoke<RawgGame[]>("search_rawg", { query });
}

/** Trigger Steam/Epic indexing and return summary counts. */
export async function indexNow(): Promise<IndexResult> {
  return invoke<IndexResult>("index_now");
}
