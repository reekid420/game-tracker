//! RAWG Video Games Database API client.

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Minimal RAWG search response payload used by this application.
#[derive(Debug, Deserialize)]
pub struct RawgSearchResponse {
    /// Search matches returned by RAWG.
    pub results: Vec<RawgGame>,
}

/// RAWG game payload fields used for enrichment and UI display.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RawgGame {
    pub id: i32,
    pub name: String,
    pub background_image: Option<String>,
    pub released: Option<String>,
    pub genres: Vec<Genre>,
    #[serde(default)]
    pub description_raw: Option<String>,
}

/// Genre entry from RAWG game metadata.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Genre {
    pub name: String,
}

/// Thin async client for RAWG game search/details endpoints.
pub struct RawgClient {
    client: Client,
    api_key: String,
}

impl RawgClient {
    /// Create a new RAWG client from an API key.
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Search RAWG by title and return up to five candidate matches.
    pub async fn search_game(
        &self,
        query: &str,
    ) -> Result<Vec<RawgGame>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.rawg.io/api/games?key={}&search={}&page_size=5",
            self.api_key,
            urlencoding::encode(query)
        );

        let response: RawgSearchResponse =
            self.client.get(&url).send().await?.json().await?;

        Ok(response.results)
    }

    /// Fetch complete details for a RAWG game id.
    pub async fn get_game_details(
        &self,
        game_id: i32,
    ) -> Result<RawgGame, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.rawg.io/api/games/{}?key={}",
            game_id, self.api_key
        );

        let game: RawgGame = self.client.get(&url).send().await?.json().await?;

        Ok(game)
    }
}
