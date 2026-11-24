use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::naming::nfo::generate_movie_nfo;
use crate::providers::engine::MetadataEngine;
use crate::providers::imdb::db::{self as imdb_db, ImdbSyncStatus};
use crate::providers::imdb::indexer::{self as imdb_indexer, DatasetUpdateInfo};
use crate::providers::imdb::ImdbLocalProvider;
use crate::providers::omdb::OmdbProvider;
use crate::providers::ordering::{calculate_episode_remap, EpisodeRemapItem};
use crate::providers::tmdb::TmdbProvider;
use crate::providers::tvdb::TvdbProvider;
use crate::providers::{
    EpisodeOrder, MetadataProvider, MovieSearchQuery, MovieSearchResult, ProviderIdMap,
    TvSearchQuery, TvSearchResult,
};
use crate::{DbState, ImdbDbState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigDto {
    pub provider_id: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub pin_or_token: Option<String>,
    pub rate_limit_per_sec: i32,
}

#[tauri::command]
pub fn get_provider_configs(state: State<'_, DbState>) -> Result<Vec<ProviderConfigDto>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT provider_id, enabled, api_key, pin_or_token, rate_limit_per_sec FROM provider_configs")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ProviderConfigDto {
                provider_id: row.get(0)?,
                enabled: row.get::<_, i32>(1)? == 1,
                api_key: row.get(2)?,
                pin_or_token: row.get(3)?,
                rate_limit_per_sec: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
pub fn set_provider_config(
    provider_id: String,
    enabled: bool,
    api_key: Option<String>,
    pin_or_token: Option<String>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"
        INSERT INTO provider_configs (provider_id, enabled, api_key, pin_or_token, updated_at)
        VALUES (?1, ?2, ?3, ?4, datetime('now'))
        ON CONFLICT(provider_id) DO UPDATE SET
            enabled = excluded.enabled,
            api_key = excluded.api_key,
            pin_or_token = excluded.pin_or_token,
            updated_at = datetime('now')
        "#,
        params![provider_id, if enabled { 1 } else { 0 }, api_key, pin_or_token],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_field_priorities(state: State<'_, DbState>) -> Result<HashMap<String, Vec<String>>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    Ok(MetadataEngine::load_priorities(&conn))
}

#[tauri::command]
pub fn set_field_priority(
    field_name: String,
    priority_order: Vec<String>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let json_val = serde_json::to_string(&priority_order).map_err(|e| e.to_string())?;
    conn.execute(
        r#"
        INSERT INTO field_priorities (field_name, priority_order)
        VALUES (?1, ?2)
        ON CONFLICT(field_name) DO UPDATE SET priority_order = excluded.priority_order
        "#,
        params![field_name, json_val],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn build_engine(
    conn: &rusqlite::Connection,
    imdb_conn: Arc<std::sync::Mutex<rusqlite::Connection>>,
) -> MetadataEngine {
    let mut configs = HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT provider_id, enabled, api_key, pin_or_token FROM provider_configs") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let pid: String = row.get(0)?;
            let enabled: i32 = row.get(1)?;
            let key: Option<String> = row.get(2)?;
            let pin: Option<String> = row.get(3)?;
            Ok((pid, (enabled == 1, key, pin)))
        }) {
            for r in rows.flatten() {
                configs.insert(r.0, r.1);
            }
        }
    }

    let mut providers: Vec<Arc<dyn MetadataProvider>> = Vec::new();

    // 1. IMDb Local
    let (imdb_enabled, _, _) = configs.get("imdb").cloned().unwrap_or((true, None, None));
    if imdb_enabled {
        providers.push(Arc::new(ImdbLocalProvider::new(imdb_conn)));
    }

    // 2. TMDb
    let (tmdb_enabled, tmdb_key, _) = configs.get("tmdb").cloned().unwrap_or((false, None, None));
    if tmdb_enabled {
        providers.push(Arc::new(TmdbProvider::new(tmdb_key)));
    }

    // 3. TVDb
    let (tvdb_enabled, tvdb_key, tvdb_pin) = configs.get("tvdb").cloned().unwrap_or((false, None, None));
    if tvdb_enabled {
        providers.push(Arc::new(TvdbProvider::new(tvdb_key, tvdb_pin)));
    }

    // 4. OMDb
    let (omdb_enabled, omdb_key, _) = configs.get("omdb").cloned().unwrap_or((false, None, None));
    if omdb_enabled {
        providers.push(Arc::new(OmdbProvider::new(omdb_key)));
    }

    MetadataEngine::new(providers)
}

#[tauri::command]
pub async fn search_metadata_movie(
    title: String,
    year: Option<u16>,
    state: State<'_, DbState>,
    imdb_state: State<'_, ImdbDbState>,
) -> Result<Vec<MovieSearchResult>, String> {
    let engine = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        build_engine(&conn, imdb_state.0.clone())
    };

    let query = MovieSearchQuery {
        title,
        year,
        limit: Some(15),
    };

    Ok(engine.search_movies(&query).await)
}

#[tauri::command]
pub async fn search_metadata_tv(
    title: String,
    year: Option<u16>,
    state: State<'_, DbState>,
    imdb_state: State<'_, ImdbDbState>,
) -> Result<Vec<TvSearchResult>, String> {
    let engine = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        build_engine(&conn, imdb_state.0.clone())
    };

    let query = TvSearchQuery {
        title,
        year,
        limit: Some(15),
    };

    Ok(engine.search_tv(&query).await)
}

#[tauri::command]
pub async fn scrape_and_save_movie(
    movie_id: i64,
    provider_ids: ProviderIdMap,
    state: State<'_, DbState>,
    imdb_state: State<'_, ImdbDbState>,
) -> Result<(), String> {
    let (engine, priorities, folder_path) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let engine = build_engine(&conn, imdb_state.0.clone());
        let priorities = MetadataEngine::load_priorities(&conn);
        let folder: String = conn
            .query_row("SELECT folder_path FROM movies WHERE id = ?1", params![movie_id], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        (engine, priorities, folder)
    };

    let details = engine
        .get_merged_movie_details(&provider_ids, &priorities)
        .await
        .map_err(|e| e.to_string())?;

    // 1. Generate and write movie.nfo per Jellyfin standards
    let nfo_content = generate_movie_nfo(&details);
    let nfo_path = Path::new(&folder_path).join("movie.nfo");
    std::fs::write(&nfo_path, nfo_content.as_bytes()).map_err(|e| format!("Failed to write NFO: {}", e))?;

    // 2. Persist scraped fields to SQLite
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let genres_json = serde_json::to_string(&details.genres).unwrap_or_default();
    let actors_json = serde_json::to_string(&details.actors).unwrap_or_default();
    let writers_json = serde_json::to_string(&details.writers).unwrap_or_default();

    conn.execute(
        r#"
        UPDATE movies SET
            title = ?1,
            original_title = ?2,
            year = ?3,
            plot = ?4,
            tagline = ?5,
            runtime = ?6,
            mpaa = ?7,
            studio = ?8,
            director = ?9,
            writers = ?10,
            actors_json = ?11,
            genres = ?12,
            poster_url = ?13,
            backdrop_url = ?14,
            imdb_id = COALESCE(?15, imdb_id),
            tmdb_id = COALESCE(?16, tmdb_id),
            tvdb_id = COALESCE(?17, tvdb_id),
            community_rating = ?18,
            vote_count = ?19,
            has_nfo = 1,
            scan_status = 'matched',
            updated_at = datetime('now')
        WHERE id = ?20
        "#,
        params![
            details.title,
            details.original_title,
            details.year.map(|y| y as i32),
            details.overview,
            details.tagline,
            details.runtime.map(|r| r as i32),
            details.mpaa,
            details.studio,
            details.director,
            writers_json,
            actors_json,
            genres_json,
            details.poster_url,
            details.backdrop_url,
            details.provider_ids.imdb_id,
            details.provider_ids.tmdb_id.map(|id| id as i64),
            details.provider_ids.tvdb_id.map(|id| id as i64),
            details.rating,
            details.vote_count.map(|v| v as i64),
            movie_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn preview_episode_remap(
    tv_show_id: i64,
    target_order: String,
    state: State<'_, DbState>,
    imdb_state: State<'_, ImdbDbState>,
) -> Result<Vec<EpisodeRemapItem>, String> {
    let order = EpisodeOrder::from_str_loose(&target_order);

    let (show, episodes, engine, priorities) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let show = crate::db::queries::get_tv_show_by_id(&conn, tv_show_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Show not found".to_string())?;

        let episodes = crate::db::queries::get_episodes_for_show(&conn, tv_show_id)
            .map_err(|e| e.to_string())?;

        let engine = build_engine(&conn, imdb_state.0.clone());
        let priorities = MetadataEngine::load_priorities(&conn);
        (show, episodes, engine, priorities)
    };

    let ids = ProviderIdMap {
        imdb_id: show.imdb_id,
        tmdb_id: show.tmdb_id.map(|id| id as u32),
        tvdb_id: show.tvdb_id.map(|id| id as u32),
    };

    let show_details = engine
        .get_merged_tv_details(&ids, order, &priorities)
        .await
        .unwrap_or_else(|_| crate::providers::TvShowDetails {
            provider_ids: ids.clone(),
            title: show.title.clone(),
            original_title: None,
            overview: None,
            year: show.year.map(|y| y as u16),
            status: None,
            genres: Vec::new(),
            mpaa: None,
            studio: None,
            actors: Vec::new(),
            poster_url: None,
            backdrop_url: None,
            banner_url: None,
            rating: None,
            vote_count: None,
            episodes: Vec::new(),
        });

    let remap_preview = calculate_episode_remap(&episodes, order, &show_details);
    Ok(remap_preview)
}

#[tauri::command]
pub async fn scrape_and_save_tv(
    tv_show_id: i64,
    provider_ids: ProviderIdMap,
    target_order: String,
    state: State<'_, DbState>,
    imdb_state: State<'_, ImdbDbState>,
) -> Result<(), String> {
    let order = EpisodeOrder::from_str_loose(&target_order);

    let (show, episodes, engine, priorities) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let show = crate::db::queries::get_tv_show_by_id(&conn, tv_show_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Show not found".to_string())?;
        let episodes = crate::db::queries::get_episodes_for_show(&conn, tv_show_id)
            .map_err(|e| e.to_string())?;
        let engine = build_engine(&conn, imdb_state.0.clone());
        let priorities = MetadataEngine::load_priorities(&conn);
        (show, episodes, engine, priorities)
    };

    let show_details = engine
        .get_merged_tv_details(&provider_ids, order, &priorities)
        .await
        .map_err(|e| e.to_string())?;

    // 1. Write tvshow.nfo per Jellyfin standards
    let nfo_content = crate::naming::nfo::generate_tvshow_nfo(&show_details, order.as_str());
    let nfo_path = Path::new(&show.folder_path).join("tvshow.nfo");
    let _ = std::fs::write(&nfo_path, nfo_content.as_bytes());

    // 2. Calculate episode remap and write episode NFOs
    let remap_items = calculate_episode_remap(&episodes, order, &show_details);

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    for remap in &remap_items {
        let ep_detail = show_details.episodes.iter().find(|e| {
            e.season == remap.target_season as u32 && e.episode == remap.target_episode as u32
        });

        if let Some(detail) = ep_detail {
            let ep_nfo = crate::naming::nfo::generate_episode_nfo(
                detail,
                &show_details.title,
                None,
                None,
            );
            let ep_file = Path::new(&remap.current_file_path);
            let nfo_name = ep_file.with_extension("nfo");
            let _ = std::fs::write(&nfo_name, ep_nfo.as_bytes());

            let _ = conn.execute(
                r#"
                UPDATE episodes SET
                    title = COALESCE(?1, title),
                    season = ?2,
                    episode = ?3,
                    plot = ?4,
                    community_rating = ?5,
                    vote_count = ?6,
                    imdb_id = COALESCE(?7, imdb_id),
                    tvdb_id = COALESCE(?8, tvdb_id),
                    tmdb_id = COALESCE(?9, tmdb_id),
                    has_nfo = 1,
                    scan_status = 'matched',
                    updated_at = datetime('now')
                WHERE id = ?10
                "#,
                params![
                    detail.title,
                    remap.target_season,
                    remap.target_episode,
                    detail.overview,
                    detail.rating,
                    detail.vote_count.map(|v| v as i64),
                    detail.provider_ids.imdb_id,
                    detail.provider_ids.tvdb_id.map(|id| id as i64),
                    detail.provider_ids.tmdb_id.map(|id| id as i64),
                    remap.episode_id,
                ],
            );
        }
    }

    // 3. Update tv_shows row in DB
    let genres_json = serde_json::to_string(&show_details.genres).unwrap_or_default();
    let actors_json = serde_json::to_string(&show_details.actors).unwrap_or_default();

    conn.execute(
        r#"
        UPDATE tv_shows SET
            title = ?1,
            original_title = ?2,
            year = ?3,
            plot = ?4,
            status = ?5,
            mpaa = ?6,
            studio = ?7,
            genres = ?8,
            actors_json = ?9,
            poster_url = ?10,
            backdrop_url = ?11,
            banner_url = ?12,
            episode_order_mode = ?13,
            imdb_id = COALESCE(?14, imdb_id),
            tvdb_id = COALESCE(?15, tvdb_id),
            tmdb_id = COALESCE(?16, tmdb_id),
            community_rating = ?17,
            vote_count = ?18,
            has_nfo = 1,
            scan_status = 'matched',
            updated_at = datetime('now')
        WHERE id = ?19
        "#,
        params![
            show_details.title,
            show_details.original_title,
            show_details.year.map(|y| y as i32),
            show_details.overview,
            show_details.status,
            show_details.mpaa,
            show_details.studio,
            genres_json,
            actors_json,
            show_details.poster_url,
            show_details.backdrop_url,
            show_details.banner_url,
            order.as_str(),
            show_details.provider_ids.imdb_id,
            show_details.provider_ids.tvdb_id.map(|id| id as i64),
            show_details.provider_ids.tmdb_id.map(|id| id as i64),
            show_details.rating,
            show_details.vote_count.map(|v| v as i64),
            tv_show_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn check_imdb_updates(
    imdb_state: State<'_, ImdbDbState>,
) -> Result<Vec<DatasetUpdateInfo>, String> {
    let local_headers = {
        let conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
        let mut map = HashMap::new();
        if let Ok(mut stmt) = conn.prepare("SELECT dataset_name, last_synced_at, last_modified_header FROM imdb_sync_status") {
            if let Ok(rows) = stmt.query_map([], |r| {
                let name: String = r.get(0)?;
                let synced: Option<String> = r.get(1)?;
                let header: Option<String> = r.get(2)?;
                Ok((name, (synced, header)))
            }) {
                for item in rows.flatten() {
                    map.insert(item.0, item.1);
                }
            }
        }
        map
    };

    imdb_indexer::check_imdb_upstream_updates(&local_headers).await
}

#[tauri::command]
pub fn get_imdb_sync_status(
    imdb_state: State<'_, ImdbDbState>,
) -> Result<Vec<ImdbSyncStatus>, String> {
    let conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
    imdb_db::get_sync_status(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn index_imdb_from_local(
    folder_path: String,
    imdb_state: State<'_, ImdbDbState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let dir = PathBuf::from(&folder_path);

    let basics = dir.join("title.basics.tsv.gz");
    let ratings = dir.join("title.ratings.tsv.gz");
    let episodes = dir.join("title.episode.tsv.gz");

    if !basics.exists() {
        return Err(format!("title.basics.tsv.gz not found in {}", folder_path));
    }

    let mut indexed_summary = Vec::new();

    // 1. Index basics
    {
        let mut conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
        let count = imdb_indexer::index_basics_tsv_gz(&mut conn, &basics, Some(&app_handle))?;
        indexed_summary.push(format!("{} titles", count));
    }

    // 2. Index ratings (if present)
    if ratings.exists() {
        let mut conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
        let count = imdb_indexer::index_ratings_tsv_gz(&mut conn, &ratings, Some(&app_handle))?;
        indexed_summary.push(format!("{} ratings", count));
    }

    // 3. Index episodes (if present)
    if episodes.exists() {
        let mut conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
        let count = imdb_indexer::index_episodes_tsv_gz(&mut conn, &episodes, Some(&app_handle))?;
        indexed_summary.push(format!("{} episodes", count));
    }

    Ok(format!("Successfully indexed: {}", indexed_summary.join(", ")))
}

#[tauri::command]
pub fn is_imdb_installed(imdb_state: State<'_, ImdbDbState>) -> Result<bool, String> {
    let conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
    Ok(crate::providers::imdb::downloader::is_imdb_installed(&conn))
}

#[tauri::command]
pub fn is_imdb_stale(imdb_state: State<'_, ImdbDbState>) -> Result<bool, String> {
    let conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
    Ok(crate::providers::imdb::downloader::is_imdb_stale(&conn))
}

#[tauri::command]
pub fn is_imdb_syncing() -> Result<bool, String> {
    Ok(crate::providers::imdb::downloader::is_downloading())
}

#[tauri::command]
pub async fn download_and_index_imdb(
    imdb_state: State<'_, ImdbDbState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;

    crate::providers::imdb::downloader::download_and_index_imdb_pipeline(
        app_data_dir,
        imdb_state.0.clone(),
        Some(app_handle),
    )
    .await
}

#[tauri::command]
pub async fn auto_refresh_imdb_if_stale(
    db_state: State<'_, DbState>,
    imdb_state: State<'_, ImdbDbState>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    let auto_refresh_enabled = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        crate::db::queries::get_setting(&conn, "imdb_auto_refresh_enabled")
            .ok()
            .flatten()
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(true)
    };

    if !auto_refresh_enabled {
        log::info!("IMDb auto-refresh is disabled in settings; skipping auto-refresh.");
        return Ok(false);
    }

    let installed = {
        let conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
        crate::providers::imdb::downloader::is_imdb_installed(&conn)
    };

    if !installed {
        log::info!("IMDb is not yet installed; skipping auto-refresh.");
        return Ok(false);
    }

    let stale = {
        let conn = imdb_state.0.lock().map_err(|e| e.to_string())?;
        crate::providers::imdb::downloader::is_imdb_stale(&conn)
    };

    if !stale {
        log::info!("IMDb dataset is up-to-date (<7 days old); skipping auto-refresh.");
        return Ok(false);
    }

    log::info!("IMDb dataset is >7 days old. Launching automatic background refresh...");
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;
    let conn_arc = imdb_state.0.clone();
    let app_h = app_handle.clone();

    tokio::spawn(async move {
        if let Err(e) = crate::providers::imdb::downloader::download_and_index_imdb_pipeline(
            app_data_dir,
            conn_arc,
            Some(app_h),
        )
        .await
        {
            log::error!("Background IMDb auto-refresh failed: {}", e);
        }
    });

    Ok(true)
}
