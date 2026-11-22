pub mod db;
pub mod indexer;

use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use crate::providers::{
    EpisodeDetails, EpisodeOrder, MetadataProvider, MovieDetails, MovieSearchQuery,
    MovieSearchResult, ProviderError, ProviderIdMap, TvSearchResult, TvShowDetails,
};

pub struct ImdbLocalProvider {
    conn: Arc<Mutex<Connection>>,
}

impl ImdbLocalProvider {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl MetadataProvider for ImdbLocalProvider {
    fn provider_id(&self) -> &'static str {
        "imdb"
    }

    fn is_available(&self) -> bool {
        if let Ok(conn) = self.conn.lock() {
            let count: i64 = conn
                .query_row("SELECT count(*) FROM imdb_titles", [], |r| r.get(0))
                .unwrap_or(0);
            count > 0
        } else {
            false
        }
    }

    async fn search_movie(
        &self,
        query: &MovieSearchQuery,
    ) -> Result<Vec<MovieSearchResult>, ProviderError> {
        let conn = self.conn.lock().map_err(|e| ProviderError::Internal(e.to_string()))?;
        let items = db::search_titles(
            &conn,
            &query.title,
            query.year,
            Some("movie"),
            query.limit.unwrap_or(10),
        )
        .map_err(|e| ProviderError::Internal(e.to_string()))?;

        let results = items
            .into_iter()
            .map(|i| MovieSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id: Some(i.tconst),
                    tmdb_id: None,
                    tvdb_id: None,
                },
                title: i.primary_title,
                year: i.start_year,
                poster_url: None, // Local TSV dataset doesn't include images
                overview: None,
                rating: i.rating,
                vote_count: i.num_votes,
            })
            .collect();

        Ok(results)
    }

    async fn search_tv(
        &self,
        query: &crate::providers::TvSearchQuery,
    ) -> Result<Vec<TvSearchResult>, ProviderError> {
        let conn = self.conn.lock().map_err(|e| ProviderError::Internal(e.to_string()))?;
        let items = db::search_titles(
            &conn,
            &query.title,
            query.year,
            Some("tvshow"),
            query.limit.unwrap_or(10),
        )
        .map_err(|e| ProviderError::Internal(e.to_string()))?;

        let results = items
            .into_iter()
            .map(|i| TvSearchResult {
                provider_ids: ProviderIdMap {
                    imdb_id: Some(i.tconst),
                    tmdb_id: None,
                    tvdb_id: None,
                },
                title: i.primary_title,
                year: i.start_year,
                poster_url: None,
                overview: None,
                rating: i.rating,
                vote_count: i.num_votes,
            })
            .collect();

        Ok(results)
    }

    async fn get_movie_details(
        &self,
        id: &ProviderIdMap,
    ) -> Result<MovieDetails, ProviderError> {
        let tconst = id
            .imdb_id
            .as_deref()
            .ok_or_else(|| ProviderError::MissingId("imdb_id required for IMDb provider".into()))?;

        let conn = self.conn.lock().map_err(|e| ProviderError::Internal(e.to_string()))?;
        let title_info = db::get_title_by_tconst(&conn, tconst)
            .map_err(|e| ProviderError::Internal(e.to_string()))?
            .ok_or_else(|| ProviderError::NotFound(format!("IMDb tconst {} not found", tconst)))?;

        Ok(MovieDetails {
            provider_ids: ProviderIdMap {
                imdb_id: Some(title_info.tconst),
                tmdb_id: None,
                tvdb_id: None,
            },
            title: title_info.primary_title,
            original_title: title_info.original_title,
            tagline: None,
            overview: None,
            year: title_info.start_year,
            runtime: title_info.runtime_minutes,
            genres: title_info.genres,
            mpaa: None,
            studio: None,
            director: None,
            writers: Vec::new(),
            actors: Vec::new(),
            poster_url: None,
            backdrop_url: None,
            rating: title_info.rating,
            vote_count: title_info.num_votes,
        })
    }

    async fn get_tv_details(
        &self,
        id: &ProviderIdMap,
        _order: EpisodeOrder,
    ) -> Result<TvShowDetails, ProviderError> {
        let tconst = id
            .imdb_id
            .as_deref()
            .ok_or_else(|| ProviderError::MissingId("imdb_id required for IMDb provider".into()))?;

        let conn = self.conn.lock().map_err(|e| ProviderError::Internal(e.to_string()))?;
        let title_info = db::get_title_by_tconst(&conn, tconst)
            .map_err(|e| ProviderError::Internal(e.to_string()))?
            .ok_or_else(|| ProviderError::NotFound(format!("IMDb tconst {} not found", tconst)))?;

        // Query all episodes for this show from imdb_episodes & imdb_titles
        let mut episodes = Vec::new();
        if let Ok(mut ep_stmt) = conn.prepare(
            r#"
            SELECT e.tconst, e.season_number, e.episode_number, t.primary_title, r.average_rating, r.num_votes
            FROM imdb_episodes e
            LEFT JOIN imdb_titles t ON e.tconst = t.tconst
            LEFT JOIN imdb_ratings r ON e.tconst = r.tconst
            WHERE e.parent_tconst = ?1 AND e.season_number IS NOT NULL AND e.episode_number IS NOT NULL
            ORDER BY e.season_number, e.episode_number
            "#,
        ) {
            if let Ok(rows) = ep_stmt.query_map(rusqlite::params![tconst], |row| {
                let ep_tconst: String = row.get(0)?;
                let s_num: i32 = row.get(1)?;
                let e_num: i32 = row.get(2)?;
                let ep_title: Option<String> = row.get(3)?;
                let rating: Option<f64> = row.get(4)?;
                let votes: Option<i64> = row.get(5)?;

                Ok(EpisodeDetails {
                    provider_ids: ProviderIdMap {
                        imdb_id: Some(ep_tconst),
                        tmdb_id: None,
                        tvdb_id: None,
                    },
                    title: ep_title,
                    season: s_num as u32,
                    episode: e_num as u32,
                    aired_date: None,
                    overview: None,
                    director: None,
                    writer: None,
                    still_url: None,
                    rating,
                    vote_count: votes.map(|v| v as u32),
                })
            }) {
                for ep in rows.flatten() {
                    episodes.push(ep);
                }
            }
        }

        Ok(TvShowDetails {
            provider_ids: ProviderIdMap {
                imdb_id: Some(title_info.tconst),
                tmdb_id: None,
                tvdb_id: None,
            },
            title: title_info.primary_title,
            original_title: title_info.original_title,
            overview: None,
            year: title_info.start_year,
            status: None,
            genres: title_info.genres,
            mpaa: None,
            studio: None,
            actors: Vec::new(),
            poster_url: None,
            backdrop_url: None,
            banner_url: None,
            rating: title_info.rating,
            vote_count: title_info.num_votes,
            episodes,
        })
    }

    async fn get_episode_details(
        &self,
        show_id: &ProviderIdMap,
        season: u32,
        episode: u32,
        _order: EpisodeOrder,
    ) -> Result<EpisodeDetails, ProviderError> {
        let parent_tconst = show_id
            .imdb_id
            .as_deref()
            .ok_or_else(|| ProviderError::MissingId("show imdb_id required".into()))?;

        let conn = self.conn.lock().map_err(|e| ProviderError::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT t.tconst, t.primary_title, r.average_rating, r.num_votes
                FROM imdb_episodes e
                JOIN imdb_titles t ON e.tconst = t.tconst
                LEFT JOIN imdb_ratings r ON e.tconst = r.tconst
                WHERE e.parent_tconst = ?1 AND e.season_number = ?2 AND e.episode_number = ?3
                LIMIT 1
                "#,
            )
            .map_err(|e| ProviderError::Internal(e.to_string()))?;

        let mut rows = stmt
            .query(rusqlite::params![parent_tconst, season as i32, episode as i32])
            .map_err(|e| ProviderError::Internal(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| ProviderError::Internal(e.to_string()))? {
            let tconst: String = row.get(0).map_err(|e| ProviderError::Internal(e.to_string()))?;
            let title: String = row.get(1).map_err(|e| ProviderError::Internal(e.to_string()))?;
            let rating: Option<f64> = row.get(2).ok();
            let vote_count: Option<u32> = row.get::<_, Option<i64>>(3).ok().flatten().map(|v| v as u32);

            Ok(EpisodeDetails {
                provider_ids: ProviderIdMap {
                    imdb_id: Some(tconst),
                    tmdb_id: None,
                    tvdb_id: None,
                },
                title: Some(title),
                season,
                episode,
                aired_date: None,
                overview: None,
                director: None,
                writer: None,
                still_url: None,
                rating,
                vote_count,
            })
        } else {
            Err(ProviderError::NotFound(format!(
                "Episode S{:02}E{:02} for {} not found in IMDb",
                season, episode, parent_tconst
            )))
        }
    }
}
