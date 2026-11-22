use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::providers::{
    Actor, EpisodeDetails, EpisodeOrder, MetadataProvider, MovieDetails, MovieSearchQuery,
    MovieSearchResult, ProviderError, ProviderIdMap, TvSearchQuery, TvSearchResult, TvShowDetails,
};

pub struct TmdbProvider {
    client: Client,
    api_key: Option<String>,
}

impl TmdbProvider {
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
            provider: "tmdb",
            message: "TMDb API key not configured in Settings".into(),
        })
    }

    fn img_url(path: Option<&str>) -> Option<String> {
        path.map(|p| format!("https://image.tmdb.org/t/p/original{}", p))
    }
}

#[async_trait]
impl MetadataProvider for TmdbProvider {
    fn provider_id(&self) -> &'static str {
        "tmdb"
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
            "https://api.themoviedb.org/3/search/movie?api_key={}&query={}",
            key,
            urlencoding::encode(&query.title)
        );

        if let Some(y) = query.year {
            url.push_str(&format!("&primary_release_year={}", y));
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
                provider: "tmdb",
                message: e.to_string(),
            })?;

        let empty = Vec::new();
        let results_json = resp["results"].as_array().unwrap_or(&empty);

        let mut list = Vec::new();
        for item in results_json {
            let tmdb_id = item["id"].as_u64().map(|id| id as u32);
            let title = item["title"].as_str().unwrap_or("").to_string();
            let release_date = item["release_date"].as_str().unwrap_or("");
            let year = release_date.split('-').next().and_then(|y| y.parse::<u16>().ok());
            let poster = Self::img_url(item["poster_path"].as_str());
            let overview = item["overview"].as_str().map(|s| s.to_string());
            let rating = item["vote_average"].as_f64();
            let vote_count = item["vote_count"].as_u64().map(|v| v as u32);

            list.push(MovieSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id: None,
                    tmdb_id,
                    tvdb_id: None,
                },
                title,
                year,
                poster_url: poster,
                overview,
                rating,
                vote_count,
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
            "https://api.themoviedb.org/3/search/tv?api_key={}&query={}",
            key,
            urlencoding::encode(&query.title)
        );

        if let Some(y) = query.year {
            url.push_str(&format!("&first_air_date_year={}", y));
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
                provider: "tmdb",
                message: e.to_string(),
            })?;

        let empty = Vec::new();
        let results_json = resp["results"].as_array().unwrap_or(&empty);

        let mut list = Vec::new();
        for item in results_json {
            let tmdb_id = item["id"].as_u64().map(|id| id as u32);
            let title = item["name"].as_str().unwrap_or("").to_string();
            let air_date = item["first_air_date"].as_str().unwrap_or("");
            let year = air_date.split('-').next().and_then(|y| y.parse::<u16>().ok());
            let poster = Self::img_url(item["poster_path"].as_str());
            let overview = item["overview"].as_str().map(|s| s.to_string());
            let rating = item["vote_average"].as_f64();
            let vote_count = item["vote_count"].as_u64().map(|v| v as u32);

            list.push(TvSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id: None,
                    tmdb_id,
                    tvdb_id: None,
                },
                title,
                year,
                poster_url: poster,
                overview,
                rating,
                vote_count,
            });
        }

        Ok(list)
    }

    async fn get_movie_details(
        &self,
        id: &ProviderIdMap,
    ) -> Result<MovieDetails, ProviderError> {
        let key = self.check_key()?;
        let tmdb_id = id
            .tmdb_id
            .ok_or_else(|| ProviderError::MissingId("tmdb_id required for TMDb provider".into()))?;

        let url = format!(
            "https://api.themoviedb.org/3/movie/{}?api_key={}&append_to_response=credits,release_dates",
            tmdb_id, key
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
                provider: "tmdb",
                message: e.to_string(),
            })?;

        let title = resp["title"].as_str().unwrap_or("").to_string();
        let original_title = resp["original_title"].as_str().map(|s| s.to_string());
        let tagline = resp["tagline"].as_str().map(|s| s.to_string()).filter(|s| !s.is_empty());
        let overview = resp["overview"].as_str().map(|s| s.to_string());
        let release_date = resp["release_date"].as_str().unwrap_or("");
        let year = release_date.split('-').next().and_then(|y| y.parse::<u16>().ok());
        let runtime = resp["runtime"].as_u64().map(|r| r as u32);

        let mut genres = Vec::new();
        if let Some(genre_arr) = resp["genres"].as_array() {
            for g in genre_arr {
                if let Some(name) = g["name"].as_str() {
                    genres.push(name.to_string());
                }
            }
        }

        let poster = Self::img_url(resp["poster_path"].as_str());
        let backdrop = Self::img_url(resp["backdrop_path"].as_str());
        let imdb_id = resp["imdb_id"].as_str().map(|s| s.to_string());

        let mut studio = None;
        if let Some(prods) = resp["production_companies"].as_array() {
            if let Some(first) = prods.first() {
                studio = first["name"].as_str().map(|s| s.to_string());
            }
        }

        let mut director = None;
        let mut writers = Vec::new();
        let mut actors = Vec::new();

        if let Some(crew) = resp["credits"]["crew"].as_array() {
            for c in crew {
                let job = c["job"].as_str().unwrap_or("");
                let name = c["name"].as_str().unwrap_or("");
                if job == "Director" && director.is_none() {
                    director = Some(name.to_string());
                } else if job == "Screenplay" || job == "Writer" {
                    writers.push(name.to_string());
                }
            }
        }

        if let Some(cast) = resp["credits"]["cast"].as_array() {
            for (idx, a) in cast.iter().take(20).enumerate() {
                let name = a["name"].as_str().unwrap_or("").to_string();
                let role = a["character"].as_str().map(|s| s.to_string());
                let thumb = Self::img_url(a["profile_path"].as_str());
                actors.push(Actor {
                    name,
                    role,
                    order: idx as u32,
                    thumb,
                });
            }
        }

        let rating = resp["vote_average"].as_f64();
        let vote_count = resp["vote_count"].as_u64().map(|v| v as u32);

        Ok(MovieDetails {
            provider_ids: ProviderIdMap {
                imdb_id,
                tmdb_id: Some(tmdb_id),
                tvdb_id: None,
            },
            title,
            original_title,
            tagline,
            overview,
            year,
            runtime,
            genres,
            mpaa: None,
            studio,
            director,
            writers,
            actors,
            poster_url: poster,
            backdrop_url: backdrop,
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
        let tmdb_id = id
            .tmdb_id
            .ok_or_else(|| ProviderError::MissingId("tmdb_id required for TMDb provider".into()))?;

        let url = format!(
            "https://api.themoviedb.org/3/tv/{}?api_key={}&append_to_response=credits,content_ratings",
            tmdb_id, key
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
                provider: "tmdb",
                message: e.to_string(),
            })?;

        let title = resp["name"].as_str().unwrap_or("").to_string();
        let original_title = resp["original_name"].as_str().map(|s| s.to_string());
        let overview = resp["overview"].as_str().map(|s| s.to_string());
        let air_date = resp["first_air_date"].as_str().unwrap_or("");
        let year = air_date.split('-').next().and_then(|y| y.parse::<u16>().ok());
        let status = resp["status"].as_str().map(|s| s.to_string());

        let mut genres = Vec::new();
        if let Some(genre_arr) = resp["genres"].as_array() {
            for g in genre_arr {
                if let Some(name) = g["name"].as_str() {
                    genres.push(name.to_string());
                }
            }
        }

        let poster = Self::img_url(resp["poster_path"].as_str());
        let backdrop = Self::img_url(resp["backdrop_path"].as_str());

        let mut studio = None;
        if let Some(networks) = resp["networks"].as_array() {
            if let Some(first) = networks.first() {
                studio = first["name"].as_str().map(|s| s.to_string());
            }
        }

        let mut actors = Vec::new();
        if let Some(cast) = resp["credits"]["cast"].as_array() {
            for (idx, a) in cast.iter().take(20).enumerate() {
                let name = a["name"].as_str().unwrap_or("").to_string();
                let role = a["character"].as_str().map(|s| s.to_string());
                let thumb = Self::img_url(a["profile_path"].as_str());
                actors.push(Actor {
                    name,
                    role,
                    order: idx as u32,
                    thumb,
                });
            }
        }

        let rating = resp["vote_average"].as_f64();
        let vote_count = resp["vote_count"].as_u64().map(|v| v as u32);

        Ok(TvShowDetails {
            provider_ids: ProviderIdMap {
                imdb_id: None,
                tmdb_id: Some(tmdb_id),
                tvdb_id: None,
            },
            title,
            original_title,
            overview,
            year,
            status,
            genres,
            mpaa: None,
            studio,
            actors,
            poster_url: poster,
            backdrop_url: backdrop,
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
        let tmdb_id = show_id
            .tmdb_id
            .ok_or_else(|| ProviderError::MissingId("tmdb_id required for episode details".into()))?;

        let url = format!(
            "https://api.themoviedb.org/3/tv/{}/season/{}/episode/{}?api_key={}",
            tmdb_id, season, episode, key
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
                provider: "tmdb",
                message: e.to_string(),
            })?;

        let title = resp["name"].as_str().map(|s| s.to_string());
        let aired_date = resp["air_date"].as_str().map(|s| s.to_string());
        let overview = resp["overview"].as_str().map(|s| s.to_string());
        let still_url = Self::img_url(resp["still_path"].as_str());
        let rating = resp["vote_average"].as_f64();
        let vote_count = resp["vote_count"].as_u64().map(|v| v as u32);

        let mut director = None;
        let mut writer = None;
        if let Some(crew) = resp["crew"].as_array() {
            for c in crew {
                let job = c["job"].as_str().unwrap_or("");
                let name = c["name"].as_str().unwrap_or("");
                if job == "Director" && director.is_none() {
                    director = Some(name.to_string());
                } else if (job == "Writer" || job == "Teleplay") && writer.is_none() {
                    writer = Some(name.to_string());
                }
            }
        }

        Ok(EpisodeDetails {
            provider_ids: ProviderIdMap {
                imdb_id: None,
                tmdb_id: resp["id"].as_u64().map(|id| id as u32),
                tvdb_id: None,
            },
            title,
            season,
            episode,
            aired_date,
            overview,
            director,
            writer,
            still_url,
            rating,
            vote_count,
        })
    }
}
