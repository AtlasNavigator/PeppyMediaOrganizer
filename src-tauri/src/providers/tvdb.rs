use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::sync::Mutex;

use crate::providers::{
    EpisodeDetails, EpisodeOrder, MetadataProvider, MovieDetails, MovieSearchQuery,
    MovieSearchResult, ProviderError, ProviderIdMap, TvSearchQuery, TvSearchResult, TvShowDetails,
};

pub struct TvdbProvider {
    client: Client,
    api_key: Option<String>,
    pin: Option<String>,
    token: Mutex<Option<String>>,
}

impl TvdbProvider {
    pub fn new(api_key: Option<String>, pin: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("PeppyMediaOrganizer/0.1.0")
                .build()
                .unwrap_or_default(),
            api_key,
            pin,
            token: Mutex::new(None),
        }
    }

    fn check_key(&self) -> Result<&str, ProviderError> {
        self.api_key.as_deref().ok_or(ProviderError::Api {
            provider: "tvdb",
            message: "TheTVDB API key not configured in Settings".into(),
        })
    }

    async fn get_token(&self) -> Result<String, ProviderError> {
        if let Ok(guard) = self.token.lock() {
            if let Some(ref t) = *guard {
                return Ok(t.clone());
            }
        }

        let api_key = self.check_key()?;
        let pin = self.pin.as_deref().unwrap_or("");

        let body = serde_json::json!({
            "apikey": api_key,
            "pin": pin
        });

        let resp: Value = self
            .client
            .post("https://api4.thetvdb.com/v4/login")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "tvdb",
                message: format!("Login failed: {}", e),
            })?;

        let token = resp["data"]["token"]
            .as_str()
            .ok_or_else(|| ProviderError::Api {
                provider: "tvdb",
                message: "Authentication token missing in response".into(),
            })?
            .to_string();

        if let Ok(mut guard) = self.token.lock() {
            *guard = Some(token.clone());
        }

        Ok(token)
    }

    fn img_url(path: Option<&str>) -> Option<String> {
        path.map(|p| {
            if p.starts_with("http") {
                p.to_string()
            } else {
                format!("https://artworks.thetvdb.com{}", p)
            }
        })
    }
}

#[async_trait]
impl MetadataProvider for TvdbProvider {
    fn provider_id(&self) -> &'static str {
        "tvdb"
    }

    fn is_available(&self) -> bool {
        self.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    async fn search_movie(
        &self,
        query: &MovieSearchQuery,
    ) -> Result<Vec<MovieSearchResult>, ProviderError> {
        let token = self.get_token().await?;
        let mut url = format!(
            "https://api4.thetvdb.com/v4/search?query={}&type=movie",
            urlencoding::encode(&query.title)
        );

        if let Some(y) = query.year {
            url.push_str(&format!("&year={}", y));
        }

        let resp: Value = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "tvdb",
                message: e.to_string(),
            })?;

        let empty = Vec::new();
        let data = resp["data"].as_array().unwrap_or(&empty);

        let mut list = Vec::new();
        for item in data {
            let tvdb_id = item["tvdb_id"]
                .as_str()
                .and_then(|s| s.parse::<u32>().ok())
                .or_else(|| item["id"].as_u64().map(|id| id as u32));

            let title = item["name"].as_str().unwrap_or("").to_string();
            let year = item["year"].as_str().and_then(|y| y.parse::<u16>().ok());
            let poster = Self::img_url(item["image_url"].as_str());
            let overview = item["overview"].as_str().map(|s| s.to_string());

            list.push(MovieSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id: None,
                    tmdb_id: None,
                    tvdb_id,
                },
                title,
                year,
                poster_url: poster,
                overview,
                rating: None,
                vote_count: None,
            });
        }

        Ok(list)
    }

    async fn search_tv(
        &self,
        query: &TvSearchQuery,
    ) -> Result<Vec<TvSearchResult>, ProviderError> {
        let token = self.get_token().await?;
        let mut url = format!(
            "https://api4.thetvdb.com/v4/search?query={}&type=series",
            urlencoding::encode(&query.title)
        );

        if let Some(y) = query.year {
            url.push_str(&format!("&year={}", y));
        }

        let resp: Value = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "tvdb",
                message: e.to_string(),
            })?;

        let empty = Vec::new();
        let data = resp["data"].as_array().unwrap_or(&empty);

        let mut list = Vec::new();
        for item in data {
            let tvdb_id = item["tvdb_id"]
                .as_str()
                .and_then(|s| s.parse::<u32>().ok())
                .or_else(|| item["id"].as_u64().map(|id| id as u32));

            let title = item["name"].as_str().unwrap_or("").to_string();
            let year = item["year"].as_str().and_then(|y| y.parse::<u16>().ok());
            let poster = Self::img_url(item["image_url"].as_str());
            let overview = item["overview"].as_str().map(|s| s.to_string());

            list.push(TvSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id: None,
                    tmdb_id: None,
                    tvdb_id,
                },
                title,
                year,
                poster_url: poster,
                overview,
                rating: None,
                vote_count: None,
            });
        }

        Ok(list)
    }

    async fn get_movie_details(
        &self,
        id: &ProviderIdMap,
    ) -> Result<MovieDetails, ProviderError> {
        let token = self.get_token().await?;
        let tvdb_id = id
            .tvdb_id
            .ok_or_else(|| ProviderError::MissingId("tvdb_id required for TVDb movie details".into()))?;

        let url = format!("https://api4.thetvdb.com/v4/movies/{}/extended", tvdb_id);

        let resp: Value = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "tvdb",
                message: e.to_string(),
            })?;

        let data = &resp["data"];
        let title = data["name"].as_str().unwrap_or("").to_string();
        let year = data["year"].as_str().and_then(|y| y.parse::<u16>().ok());
        let runtime = data["runtime"].as_u64().map(|r| r as u32);
        let overview = data["overview"].as_str().map(|s| s.to_string());
        let poster = Self::img_url(data["image"].as_str());

        let mut genres = Vec::new();
        if let Some(genre_arr) = data["genres"].as_array() {
            for g in genre_arr {
                if let Some(n) = g["name"].as_str() {
                    genres.push(n.to_string());
                }
            }
        }

        Ok(MovieDetails {
            provider_ids: ProviderIdMap {
                imdb_id: None,
                tmdb_id: None,
                tvdb_id: Some(tvdb_id),
            },
            title,
            original_title: None,
            tagline: None,
            overview,
            year,
            runtime,
            genres,
            mpaa: None,
            studio: None,
            director: None,
            writers: Vec::new(),
            actors: Vec::new(),
            poster_url: poster,
            backdrop_url: None,
            rating: None,
            vote_count: None,
        })
    }

    async fn get_tv_details(
        &self,
        id: &ProviderIdMap,
        order: EpisodeOrder,
    ) -> Result<TvShowDetails, ProviderError> {
        let token = self.get_token().await?;
        let tvdb_id = id
            .tvdb_id
            .ok_or_else(|| ProviderError::MissingId("tvdb_id required for TVDb show details".into()))?;

        let url = format!("https://api4.thetvdb.com/v4/series/{}/extended", tvdb_id);

        let resp: Value = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "tvdb",
                message: e.to_string(),
            })?;

        let data = &resp["data"];
        let title = data["name"].as_str().unwrap_or("").to_string();
        let year = data["year"].as_str().and_then(|y| y.parse::<u16>().ok());
        let overview = data["overview"].as_str().map(|s| s.to_string());
        let status = data["status"]["name"].as_str().map(|s| s.to_string());
        let poster = Self::img_url(data["image"].as_str());

        let mut genres = Vec::new();
        if let Some(genre_arr) = data["genres"].as_array() {
            for g in genre_arr {
                if let Some(n) = g["name"].as_str() {
                    genres.push(n.to_string());
                }
            }
        }

        // Fetch episodes according to season type
        let season_type = match order {
            EpisodeOrder::Aired => "default",
            EpisodeOrder::Dvd => "dvd",
            EpisodeOrder::Streaming => "official",
            EpisodeOrder::Absolute => "absolute",
        };

        let ep_url = format!(
            "https://api4.thetvdb.com/v4/series/{}/episodes/{}",
            tvdb_id, season_type
        );

        let mut episodes = Vec::new();
        if let Ok(ep_resp) = self
            .client
            .get(&ep_url)
            .bearer_auth(&token)
            .send()
            .await
        {
            if let Ok(ep_json) = ep_resp.json::<Value>().await {
                if let Some(ep_arr) = ep_json["data"]["episodes"].as_array() {
                    for ep in ep_arr {
                        let ep_title = ep["name"].as_str().map(|s| s.to_string());
                        let season = ep["seasonNumber"].as_u64().unwrap_or(1) as u32;
                        let episode_num = ep["number"].as_u64().unwrap_or(1) as u32;
                        let aired_date = ep["aired"].as_str().map(|s| s.to_string());
                        let ep_overview = ep["overview"].as_str().map(|s| s.to_string());
                        let still_url = Self::img_url(ep["image"].as_str());

                        episodes.push(EpisodeDetails {
                            provider_ids: ProviderIdMap {
                                imdb_id: None,
                                tmdb_id: None,
                                tvdb_id: ep["id"].as_u64().map(|id| id as u32),
                            },
                            title: ep_title,
                            season,
                            episode: episode_num,
                            aired_date,
                            overview: ep_overview,
                            director: None,
                            writer: None,
                            still_url,
                            rating: None,
                            vote_count: None,
                        });
                    }
                }
            }
        }

        Ok(TvShowDetails {
            provider_ids: ProviderIdMap {
                imdb_id: None,
                tmdb_id: None,
                tvdb_id: Some(tvdb_id),
            },
            title,
            original_title: None,
            overview,
            year,
            status,
            genres,
            mpaa: None,
            studio: None,
            actors: Vec::new(),
            poster_url: poster,
            backdrop_url: None,
            banner_url: None,
            rating: None,
            vote_count: None,
            episodes,
        })
    }

    async fn get_episode_details(
        &self,
        show_id: &ProviderIdMap,
        season: u32,
        episode: u32,
        order: EpisodeOrder,
    ) -> Result<EpisodeDetails, ProviderError> {
        let show_details = self.get_tv_details(show_id, order).await?;
        show_details
            .episodes
            .into_iter()
            .find(|e| e.season == season && e.episode == episode)
            .ok_or_else(|| {
                ProviderError::NotFound(format!("Episode S{:02}E{:02} not found in TVDB", season, episode))
            })
    }
}
