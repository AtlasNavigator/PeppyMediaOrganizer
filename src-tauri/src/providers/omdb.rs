use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::providers::{
    Actor, EpisodeDetails, EpisodeOrder, MetadataProvider, MovieDetails, MovieSearchQuery,
    MovieSearchResult, ProviderError, ProviderIdMap, TvSearchQuery, TvSearchResult, TvShowDetails,
};

pub struct OmdbProvider {
    client: Client,
    api_key: Option<String>,
}

impl OmdbProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("PeppyMediaOrganizer/0.1.0")
                .build()
                .unwrap_or_default(),
            api_key,
        }
    }

    fn check_key(&self) -> Result<&str, ProviderError> {
        self.api_key.as_deref().ok_or(ProviderError::Api {
            provider: "omdb",
            message: "OMDb API key not configured in Settings".into(),
        })
    }
}

#[async_trait]
impl MetadataProvider for OmdbProvider {
    fn provider_id(&self) -> &'static str {
        "omdb"
    }

    fn is_available(&self) -> bool {
        self.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    async fn search_movie(
        &self,
        query: &MovieSearchQuery,
    ) -> Result<Vec<MovieSearchResult>, ProviderError> {
        let key = self.check_key()?;
        let mut url = format!(
            "https://www.omdbapi.com/?apikey={}&s={}&type=movie",
            key,
            urlencoding::encode(&query.title)
        );

        if let Some(y) = query.year {
            url.push_str(&format!("&y={}", y));
        }

        let resp: Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "omdb",
                message: e.to_string(),
            })?;

        if resp["Response"].as_str() != Some("True") {
            return Ok(Vec::new());
        }

        let empty = Vec::new();
        let search_arr = resp["Search"].as_array().unwrap_or(&empty);

        let mut list = Vec::new();
        for item in search_arr {
            let imdb_id = item["imdbID"].as_str().map(|s| s.to_string());
            let title = item["Title"].as_str().unwrap_or("").to_string();
            let year_str = item["Year"].as_str().unwrap_or("");
            let year = year_str.chars().take(4).collect::<String>().parse::<u16>().ok();
            let poster = item["Poster"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string());

            list.push(MovieSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id,
                    tmdb_id: None,
                    tvdb_id: None,
                },
                title,
                year,
                poster_url: poster,
                overview: None,
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
        let key = self.check_key()?;
        let mut url = format!(
            "https://www.omdbapi.com/?apikey={}&s={}&type=series",
            key,
            urlencoding::encode(&query.title)
        );

        if let Some(y) = query.year {
            url.push_str(&format!("&y={}", y));
        }

        let resp: Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "omdb",
                message: e.to_string(),
            })?;

        if resp["Response"].as_str() != Some("True") {
            return Ok(Vec::new());
        }

        let empty = Vec::new();
        let search_arr = resp["Search"].as_array().unwrap_or(&empty);

        let mut list = Vec::new();
        for item in search_arr {
            let imdb_id = item["imdbID"].as_str().map(|s| s.to_string());
            let title = item["Title"].as_str().unwrap_or("").to_string();
            let year_str = item["Year"].as_str().unwrap_or("");
            let year = year_str.chars().take(4).collect::<String>().parse::<u16>().ok();
            let poster = item["Poster"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string());

            list.push(TvSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id,
                    tmdb_id: None,
                    tvdb_id: None,
                },
                title,
                year,
                poster_url: poster,
                overview: None,
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
        let key = self.check_key()?;
        let query_param = if let Some(ref imdb) = id.imdb_id {
            format!("&i={}", imdb)
        } else {
            return Err(ProviderError::MissingId("imdb_id required for OMDb".into()));
        };

        let url = format!(
            "https://www.omdbapi.com/?apikey={}{}&plot=full",
            key, query_param
        );

        let resp: Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "omdb",
                message: e.to_string(),
            })?;

        if resp["Response"].as_str() != Some("True") {
            return Err(ProviderError::NotFound(
                resp["Error"].as_str().unwrap_or("Movie not found in OMDb").to_string(),
            ));
        }

        let title = resp["Title"].as_str().unwrap_or("").to_string();
        let year = resp["Year"]
            .as_str()
            .and_then(|y| y.chars().take(4).collect::<String>().parse::<u16>().ok());
        let mpaa = resp["Rated"].as_str().filter(|r| *r != "N/A").map(|s| s.to_string());
        let runtime = resp["Runtime"]
            .as_str()
            .and_then(|r| r.split_whitespace().next()?.parse::<u32>().ok());

        let genres = resp["Genre"]
            .as_str()
            .filter(|g| *g != "N/A")
            .map(|g| g.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let director = resp["Director"].as_str().filter(|d| *d != "N/A").map(|s| s.to_string());
        let writers: Vec<String> = resp["Writer"]
            .as_str()
            .filter(|w| *w != "N/A")
            .map(|w| w.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let mut actors = Vec::new();
        if let Some(actor_str) = resp["Actors"].as_str().filter(|a| *a != "N/A") {
            for (idx, name) in actor_str.split(',').enumerate() {
                actors.push(Actor {
                    name: name.trim().to_string(),
                    role: None,
                    order: idx as u32,
                    thumb: None,
                });
            }
        }

        let overview = resp["Plot"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string());
        let poster = resp["Poster"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string());
        let rating = resp["imdbRating"].as_str().and_then(|r| r.parse::<f64>().ok());
        let vote_count = resp["imdbVotes"]
            .as_str()
            .and_then(|v| v.replace(',', "").parse::<u32>().ok());

        Ok(MovieDetails {
            provider_ids: ProviderIdMap {
                imdb_id: resp["imdbID"].as_str().map(|s| s.to_string()),
                tmdb_id: None,
                tvdb_id: None,
            },
            title,
            original_title: None,
            tagline: None,
            overview,
            year,
            runtime,
            genres,
            mpaa,
            studio: resp["Production"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string()),
            director,
            writers,
            actors,
            poster_url: poster,
            backdrop_url: None,
            rating,
            vote_count,
        })
    }

    async fn get_tv_details(
        &self,
        id: &ProviderIdMap,
        _order: EpisodeOrder,
    ) -> Result<TvShowDetails, ProviderError> {
        let key = self.check_key()?;
        let imdb_id = id
            .imdb_id
            .as_deref()
            .ok_or_else(|| ProviderError::MissingId("imdb_id required for OMDb TV query".into()))?;

        let url = format!(
            "https://www.omdbapi.com/?apikey={}&i={}&plot=full",
            key, imdb_id
        );

        let resp: Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "omdb",
                message: e.to_string(),
            })?;

        if resp["Response"].as_str() != Some("True") {
            return Err(ProviderError::NotFound("Show not found in OMDb".into()));
        }

        let title = resp["Title"].as_str().unwrap_or("").to_string();
        let year = resp["Year"]
            .as_str()
            .and_then(|y| y.chars().take(4).collect::<String>().parse::<u16>().ok());
        let mpaa = resp["Rated"].as_str().filter(|r| *r != "N/A").map(|s| s.to_string());
        let genres = resp["Genre"]
            .as_str()
            .filter(|g| *g != "N/A")
            .map(|g| g.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let mut actors = Vec::new();
        if let Some(actor_str) = resp["Actors"].as_str().filter(|a| *a != "N/A") {
            for (idx, name) in actor_str.split(',').enumerate() {
                actors.push(Actor {
                    name: name.trim().to_string(),
                    role: None,
                    order: idx as u32,
                    thumb: None,
                });
            }
        }

        let overview = resp["Plot"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string());
        let poster = resp["Poster"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string());
        let rating = resp["imdbRating"].as_str().and_then(|r| r.parse::<f64>().ok());
        let vote_count = resp["imdbVotes"]
            .as_str()
            .and_then(|v| v.replace(',', "").parse::<u32>().ok());

        Ok(TvShowDetails {
            provider_ids: ProviderIdMap {
                imdb_id: Some(imdb_id.to_string()),
                tmdb_id: None,
                tvdb_id: None,
            },
            title,
            original_title: None,
            overview,
            year,
            status: None,
            genres,
            mpaa,
            studio: None,
            actors,
            poster_url: poster,
            backdrop_url: None,
            banner_url: None,
            rating,
            vote_count,
            episodes: Vec::new(),
        })
    }

    async fn get_episode_details(
        &self,
        show_id: &ProviderIdMap,
        season: u32,
        episode: u32,
        _order: EpisodeOrder,
    ) -> Result<EpisodeDetails, ProviderError> {
        let key = self.check_key()?;
        let imdb_id = show_id
            .imdb_id
            .as_deref()
            .ok_or_else(|| ProviderError::MissingId("imdb_id required for OMDb episode".into()))?;

        let url = format!(
            "https://www.omdbapi.com/?apikey={}&i={}&Season={}&Episode={}",
            key, imdb_id, season, episode
        );

        let resp: Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::Api {
                provider: "omdb",
                message: e.to_string(),
            })?;

        let title = resp["Title"].as_str().map(|s| s.to_string());
        let aired_date = resp["Released"].as_str().filter(|r| *r != "N/A").map(|s| s.to_string());
        let overview = resp["Plot"].as_str().filter(|p| *p != "N/A").map(|s| s.to_string());
        let director = resp["Director"].as_str().filter(|d| *d != "N/A").map(|s| s.to_string());
        let writer = resp["Writer"].as_str().filter(|w| *w != "N/A").map(|s| s.to_string());
        let rating = resp["imdbRating"].as_str().and_then(|r| r.parse::<f64>().ok());
        let vote_count = resp["imdbVotes"]
            .as_str()
            .and_then(|v| v.replace(',', "").parse::<u32>().ok());

        Ok(EpisodeDetails {
            provider_ids: ProviderIdMap {
                imdb_id: resp["imdbID"].as_str().map(|s| s.to_string()),
                tmdb_id: None,
                tvdb_id: None,
            },
            title,
            season,
            episode,
            aired_date,
            overview,
            director,
            writer,
            still_url: None,
            rating,
            vote_count,
        })
    }
}
