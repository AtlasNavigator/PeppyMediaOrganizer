use std::collections::HashMap;
use std::sync::Arc;
use rusqlite::Connection;

use crate::providers::{
    Actor, EpisodeOrder, MetadataProvider, MovieDetails, MovieSearchQuery, MovieSearchResult,
    ProviderError, ProviderIdMap, TvSearchQuery, TvSearchResult, TvShowDetails,
};

pub struct MetadataEngine {
    providers: HashMap<&'static str, Arc<dyn MetadataProvider>>,
}

impl MetadataEngine {
    pub fn new(providers: Vec<Arc<dyn MetadataProvider>>) -> Self {
        let mut map = HashMap::new();
        for p in providers {
            map.insert(p.provider_id(), p);
        }
        Self { providers: map }
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn MetadataProvider>> {
        self.providers.get(name).cloned()
    }

    /// Fetches field priorities from database
    pub fn load_priorities(conn: &Connection) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        if let Ok(mut stmt) = conn.prepare("SELECT field_name, priority_order FROM field_priorities") {
            if let Ok(rows) = stmt.query_map([], |row| {
                let field: String = row.get(0)?;
                let order_json: String = row.get(1)?;
                let order: Vec<String> = serde_json::from_str(&order_json).unwrap_or_default();
                Ok((field, order))
            }) {
                for r in rows.flatten() {
                    map.insert(r.0, r.1);
                }
            }
        }
        map
    }

    /// Search movies across enabled providers
    pub async fn search_movies(
        &self,
        query: &MovieSearchQuery,
    ) -> Vec<MovieSearchResult> {
        let mut all_results = Vec::new();

        // Search in parallel across available providers
        let mut handles = Vec::new();
        for provider in self.providers.values() {
            if provider.is_available() {
                let p = provider.clone();
                let q = query.clone();
                handles.push(tokio::spawn(async move {
                    p.search_movie(&q).await.unwrap_or_default()
                }));
            }
        }

        for h in handles {
            if let Ok(res) = h.await {
                all_results.extend(res);
            }
        }

        all_results
    }

    /// Search TV shows across enabled providers
    pub async fn search_tv(
        &self,
        query: &TvSearchQuery,
    ) -> Vec<TvSearchResult> {
        let mut all_results = Vec::new();

        let mut handles = Vec::new();
        for provider in self.providers.values() {
            if provider.is_available() {
                let p = provider.clone();
                let q = query.clone();
                handles.push(tokio::spawn(async move {
                    p.search_tv(&q).await.unwrap_or_default()
                }));
            }
        }

        for h in handles {
            if let Ok(res) = h.await {
                all_results.extend(res);
            }
        }

        all_results
    }

    /// Merges movie details across multiple providers according to field priorities
    pub async fn get_merged_movie_details(
        &self,
        ids: &ProviderIdMap,
        priorities: &HashMap<String, Vec<String>>,
    ) -> Result<MovieDetails, ProviderError> {
        let mut provider_results: HashMap<&'static str, MovieDetails> = HashMap::new();

        for (&name, provider) in &self.providers {
            if provider.is_available() {
                if let Ok(details) = provider.get_movie_details(ids).await {
                    provider_results.insert(name, details);
                }
            }
        }

        if provider_results.is_empty() {
            return Err(ProviderError::NotFound("No provider returned metadata".into()));
        }

        // Helper to get field value by priority
        let get_by_priority = |field: &str, extractor: &dyn Fn(&MovieDetails) -> Option<String>| -> Option<String> {
            let default_priority = vec!["tmdb".to_string(), "imdb".to_string(), "omdb".to_string(), "tvdb".to_string()];
            let order = priorities.get(field).unwrap_or(&default_priority);
            for p in order {
                if let Some(details) = provider_results.get(p.as_str()) {
                    if let Some(val) = extractor(details) {
                        if !val.trim().is_empty() {
                            return Some(val);
                        }
                    }
                }
            }
            None
        };

        // Merge Provider IDs
        let mut merged_ids = ids.clone();
        for details in provider_results.values() {
            if merged_ids.imdb_id.is_none() && details.provider_ids.imdb_id.is_some() {
                merged_ids.imdb_id = details.provider_ids.imdb_id.clone();
            }
            if merged_ids.tmdb_id.is_none() && details.provider_ids.tmdb_id.is_some() {
                merged_ids.tmdb_id = details.provider_ids.tmdb_id;
            }
            if merged_ids.tvdb_id.is_none() && details.provider_ids.tvdb_id.is_some() {
                merged_ids.tvdb_id = details.provider_ids.tvdb_id;
            }
        }

        let title = get_by_priority("title", &|d| Some(d.title.clone())).unwrap_or_else(|| "Unknown Title".into());
        let original_title = get_by_priority("title", &|d| d.original_title.clone());
        let tagline = get_by_priority("tagline", &|d| d.tagline.clone());
        let overview = get_by_priority("overview", &|d| d.overview.clone());
        let mpaa = get_by_priority("certification", &|d| d.mpaa.clone());
        let studio = get_by_priority("studio", &|d| d.studio.clone());
        let director = get_by_priority("crew", &|d| d.director.clone());
        let poster_url = get_by_priority("posters", &|d| d.poster_url.clone());
        let backdrop_url = get_by_priority("backdrops", &|d| d.backdrop_url.clone());

        // Extract rating from prioritized provider (e.g. IMDb first)
        let rating_order = priorities.get("ratings").cloned().unwrap_or_else(|| vec!["imdb".into(), "tmdb".into(), "omdb".into()]);
        let mut rating = None;
        let mut vote_count = None;
        for p in &rating_order {
            if let Some(details) = provider_results.get(p.as_str()) {
                if details.rating.is_some() {
                    rating = details.rating;
                    vote_count = details.vote_count;
                    break;
                }
            }
        }

        // Year & runtime
        let year = provider_results.values().find_map(|d| d.year);
        let runtime = provider_results.values().find_map(|d| d.runtime);

        // Genres
        let genres = provider_results
            .values()
            .find(|d| !d.genres.is_empty())
            .map(|d| d.genres.clone())
            .unwrap_or_default();

        // Writers & Actors
        let writers = provider_results
            .values()
            .find(|d| !d.writers.is_empty())
            .map(|d| d.writers.clone())
            .unwrap_or_default();

        let actors: Vec<Actor> = provider_results
            .values()
            .find(|d| !d.actors.is_empty())
            .map(|d| d.actors.clone())
            .unwrap_or_default();

        Ok(MovieDetails {
            provider_ids: merged_ids,
            title,
            original_title,
            tagline,
            overview,
            year,
            runtime,
            genres,
            mpaa,
            studio,
            director,
            writers,
            actors,
            poster_url,
            backdrop_url,
            rating,
            vote_count,
        })
    }

    /// Merges TV show details across multiple providers according to field priorities
    pub async fn get_merged_tv_details(
        &self,
        ids: &ProviderIdMap,
        order: EpisodeOrder,
        priorities: &HashMap<String, Vec<String>>,
    ) -> Result<TvShowDetails, ProviderError> {
        let mut provider_results: HashMap<&'static str, TvShowDetails> = HashMap::new();

        let provider_check_order = ["tvdb", "tmdb", "imdb", "omdb"];
        for &name in &provider_check_order {
            if let Some(provider) = self.providers.get(name) {
                if provider.is_available() {
                    if let Ok(details) = provider.get_tv_details(ids, order).await {
                        provider_results.insert(name, details);
                    }
                }
            }
        }

        if provider_results.is_empty() {
            return Err(ProviderError::NotFound("No provider returned TV metadata".into()));
        }

        let get_by_priority = |field: &str, extractor: &dyn Fn(&TvShowDetails) -> Option<String>| -> Option<String> {
            let default_priority = vec!["tvdb".to_string(), "tmdb".to_string(), "imdb".to_string(), "omdb".to_string()];
            let order = priorities.get(field).unwrap_or(&default_priority);
            for p in order {
                if let Some(details) = provider_results.get(p.as_str()) {
                    if let Some(val) = extractor(details) {
                        if !val.trim().is_empty() {
                            return Some(val);
                        }
                    }
                }
            }
            None
        };

        let mut merged_ids = ids.clone();
        for details in provider_results.values() {
            if merged_ids.imdb_id.is_none() && details.provider_ids.imdb_id.is_some() {
                merged_ids.imdb_id = details.provider_ids.imdb_id.clone();
            }
            if merged_ids.tmdb_id.is_none() && details.provider_ids.tmdb_id.is_some() {
                merged_ids.tmdb_id = details.provider_ids.tmdb_id;
            }
            if merged_ids.tvdb_id.is_none() && details.provider_ids.tvdb_id.is_some() {
                merged_ids.tvdb_id = details.provider_ids.tvdb_id;
            }
        }

        let title = get_by_priority("title", &|d| Some(d.title.clone())).unwrap_or_else(|| "Unknown Series".into());
        let original_title = get_by_priority("title", &|d| d.original_title.clone());
        let overview = get_by_priority("overview", &|d| d.overview.clone());
        let status = get_by_priority("status", &|d| d.status.clone());
        let mpaa = get_by_priority("certification", &|d| d.mpaa.clone());
        let studio = get_by_priority("studio", &|d| d.studio.clone());
        let poster_url = get_by_priority("posters", &|d| d.poster_url.clone());
        let backdrop_url = get_by_priority("backdrops", &|d| d.backdrop_url.clone());
        let banner_url = get_by_priority("banners", &|d| d.banner_url.clone());

        let rating_order = priorities.get("ratings").cloned().unwrap_or_else(|| vec!["imdb".into(), "tmdb".into(), "tvdb".into()]);
        let mut rating = None;
        let mut vote_count = None;
        for p in &rating_order {
            if let Some(details) = provider_results.get(p.as_str()) {
                if details.rating.is_some() {
                    rating = details.rating;
                    vote_count = details.vote_count;
                    break;
                }
            }
        }

        let year = provider_results.values().find_map(|d| d.year);

        let genres = provider_results
            .values()
            .find(|d| !d.genres.is_empty())
            .map(|d| d.genres.clone())
            .unwrap_or_default();

        let actors: Vec<Actor> = provider_results
            .values()
            .find(|d| !d.actors.is_empty())
            .map(|d| d.actors.clone())
            .unwrap_or_default();

        // Episodes: prefer TVDB (handles all orders natively), then IMDb/TMDb
        let episodes = if let Some(tvdb_res) = provider_results.get("tvdb") {
            if !tvdb_res.episodes.is_empty() {
                tvdb_res.episodes.clone()
            } else {
                provider_results.values().find(|d| !d.episodes.is_empty()).map(|d| d.episodes.clone()).unwrap_or_default()
            }
        } else {
            provider_results.values().find(|d| !d.episodes.is_empty()).map(|d| d.episodes.clone()).unwrap_or_default()
        };

        Ok(TvShowDetails {
            provider_ids: merged_ids,
            title,
            original_title,
            overview,
            year,
            status,
            genres,
            mpaa,
            studio,
            actors,
            poster_url,
            backdrop_url,
            banner_url,
            rating,
            vote_count,
            episodes,
        })
    }
}
