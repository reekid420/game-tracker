//! RAWG Video Games Database API client.
//!
//! Search games, fetch details (cover, description, genres). Requires
//! `RAWG_API_KEY` in `.env`. Free tier: 20k requests/month.

use reqwest::Client;
use serde::Deserialize;

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
