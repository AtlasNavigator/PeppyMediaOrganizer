pub mod engine;
pub mod imdb;
pub mod omdb;
pub mod ordering;
pub mod tmdb;
pub mod tvdb;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Missing ID: {0}")]
    MissingId(String),
    #[error("API error from {provider}: {message}")]
    Api {
        provider: &'static str,
        message: String,
    },
    #[error("Network error: {0}")]
    Network(String),
    #[error("Internal database error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderIdMap {
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u32>,
    pub tvdb_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EpisodeOrder {
    #[default]
    Aired,
    Dvd,
    Streaming,
    Absolute,
}

impl EpisodeOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aired => "aired",
            Self::Dvd => "dvd",
            Self::Streaming => "streaming",
            Self::Absolute => "absolute",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dvd" => Self::Dvd,
            "streaming" | "digital" => Self::Streaming,
            "absolute" => Self::Absolute,
            _ => Self::Aired,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub name: String,
    pub role: Option<String>,
    pub order: u32,
    pub thumb: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieSearchQuery {
    pub title: String,
    pub year: Option<u16>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieSearchResult {
    pub provider_ids: ProviderIdMap,
    pub title: String,
    pub year: Option<u16>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub rating: Option<f64>,
    pub vote_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetails {
    pub provider_ids: ProviderIdMap,
    pub title: String,
    pub original_title: Option<String>,
    pub tagline: Option<String>,
    pub overview: Option<String>,
    pub year: Option<u16>,
    pub runtime: Option<u32>,
    pub genres: Vec<String>,
    pub mpaa: Option<String>,
    pub studio: Option<String>,
    pub director: Option<String>,
    pub writers: Vec<String>,
    pub actors: Vec<Actor>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub rating: Option<f64>,
    pub vote_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvSearchQuery {
    pub title: String,
    pub year: Option<u16>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvSearchResult {
    pub provider_ids: ProviderIdMap,
    pub title: String,
    pub year: Option<u16>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub rating: Option<f64>,
    pub vote_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeDetails {
    pub provider_ids: ProviderIdMap,
    pub title: Option<String>,
    pub season: u32,
    pub episode: u32,
    pub aired_date: Option<String>,
    pub overview: Option<String>,
    pub director: Option<String>,
    pub writer: Option<String>,
    pub still_url: Option<String>,
    pub rating: Option<f64>,
    pub vote_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvShowDetails {
    pub provider_ids: ProviderIdMap,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub year: Option<u16>,
    pub status: Option<String>,
    pub genres: Vec<String>,
    pub mpaa: Option<String>,
    pub studio: Option<String>,
    pub actors: Vec<Actor>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub banner_url: Option<String>,
    pub rating: Option<f64>,
    pub vote_count: Option<u32>,
    pub episodes: Vec<EpisodeDetails>,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn is_available(&self) -> bool;

    async fn search_movie(
        &self,
        query: &MovieSearchQuery,
    ) -> Result<Vec<MovieSearchResult>, ProviderError>;

    async fn search_tv(
        &self,
        query: &TvSearchQuery,
    ) -> Result<Vec<TvSearchResult>, ProviderError>;

    async fn get_movie_details(
        &self,
        id: &ProviderIdMap,
    ) -> Result<MovieDetails, ProviderError>;

    async fn get_tv_details(
        &self,
        id: &ProviderIdMap,
        order: EpisodeOrder,
    ) -> Result<TvShowDetails, ProviderError>;

    async fn get_episode_details(
        &self,
        show_id: &ProviderIdMap,
        season: u32,
        episode: u32,
        order: EpisodeOrder,
    ) -> Result<EpisodeDetails, ProviderError>;
}
