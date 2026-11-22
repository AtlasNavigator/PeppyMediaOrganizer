use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;
use once_cell::sync::Lazy;
use regex::Regex;
use rusqlite::{params, Connection};
use tauri::Emitter;
use walkdir::WalkDir;

use crate::db::models::{Episode, Movie, TvShow};
use crate::db::queries::{insert_episode, insert_movie, insert_tv_show, update_library_root_scan_time};
use crate::scanner::parser::{detect_sidecars, parse_filename};
use crate::scanner::{ScanProgress, ScanResult};

static SEASON_FOLDER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:season|s)\s*(\d{1,2})\b").unwrap()
});

static LEADING_EP_NUM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(\d{1,3})\b").unwrap()
});

pub const VIDEO_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "m4v", "ts", "wmv", "mov", "flv", "webm"];
pub const EXTRAS_FOLDERS: &[&str] = &[
    "trailers",
    "behind the scenes",
    "deleted scenes",
    "featurettes",
    "interviews",
    "scenes",
    "shorts",
    "extras",
];

pub fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn is_hidden_or_extra(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') || name.starts_with('_') {
            return true;
        }
        let lower = name.to_lowercase();
        if EXTRAS_FOLDERS.contains(&lower.as_str()) {
            return true;
        }
    }
    false
}

/// Scans a movie library root.
/// Expects each subfolder to represent a movie (or video files in root as fallback).
pub fn scan_movie_root(
    root_path: &str,
    root_id: i64,
    conn_mutex: &Mutex<Connection>,
    app_handle: Option<&tauri::AppHandle>,
) -> ScanResult {
    let start_time = Instant::now();
    let root = Path::new(root_path);
    let mut movies_found = 0;

    let entries: Vec<PathBuf> = match std::fs::read_dir(root) {
        Ok(read_dir) => read_dir
            .flatten()
            .map(|e| e.path())
            .filter(|p| !is_hidden_or_extra(p))
            .collect(),
        Err(_) => {
            return ScanResult {
                root_id,
                movies_found: 0,
                shows_found: 0,
                episodes_found: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
            }
        }
    };

    let total_dirs = entries.len();

    for (index, entry_path) in entries.iter().enumerate() {
        let path_str = entry_path.to_string_lossy().to_string();

        if let Some(handle) = app_handle {
            let _ = handle.emit(
                "scan:progress",
                ScanProgress {
                    scanned: index + 1,
                    total_dirs,
                    current_path: path_str.clone(),
                },
            );
        }

        let mut movie_videos: Vec<(PathBuf, u64)> = Vec::new();
        let folder_path_str: String;

        if entry_path.is_dir() {
            folder_path_str = entry_path.to_string_lossy().to_string();
            if let Ok(dir_entries) = std::fs::read_dir(entry_path) {
                for dir_entry in dir_entries.flatten() {
                    let file_path = dir_entry.path();
                    if file_path.is_file() && is_video_file(&file_path) {
                        let size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                        movie_videos.push((file_path, size));
                    }
                }
            }
        } else if entry_path.is_file() && is_video_file(entry_path) {
            folder_path_str = root_path.to_string();
            let size = std::fs::metadata(entry_path).map(|m| m.len()).unwrap_or(0);
            movie_videos.push((entry_path.clone(), size));
        } else {
            continue;
        }

        let folder_name = entry_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let folder_parsed = parse_filename(folder_name);

        for (video_path, file_size) in movie_videos {
            let video_name = video_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let video_stem = video_path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
            let file_parsed = parse_filename(video_name);

            // Inherit title, year, provider IDs from folder if available; fallback to video file
            let title = folder_parsed
                .title
                .clone()
                .or_else(|| file_parsed.title.clone())
                .unwrap_or_else(|| {
                    if entry_path.is_dir() {
                        folder_name.to_string()
                    } else {
                        video_stem.to_string()
                    }
                });

            let year = folder_parsed.year.or(file_parsed.year);
            let resolution = file_parsed.resolution.or_else(|| folder_parsed.resolution.clone());
            let imdb_id = folder_parsed.imdb_id.clone().or_else(|| file_parsed.imdb_id.clone());
            let tmdb_id = folder_parsed.tmdb_id.or(file_parsed.tmdb_id);
            let version_label = file_parsed.version_label.or_else(|| folder_parsed.version_label.clone());

            let sidecars = if entry_path.is_dir() {
                detect_sidecars(entry_path, video_stem)
            } else {
                detect_sidecars(root, video_stem)
            };

            let scan_status = if imdb_id.is_some() || tmdb_id.is_some() {
                "matched"
            } else if folder_parsed.title.is_some() || file_parsed.title.is_some() {
                "unmatched"
            } else {
                "error"
            };

            let movie = Movie {
                id: 0,
                library_root_id: root_id,
                folder_path: folder_path_str.clone(),
                file_path: video_path.to_string_lossy().to_string(),
                title,
                sort_title: None,
                year: year.map(|y| y as i32),
                resolution,
                imdb_id,
                tmdb_id: tmdb_id.map(|id| id as i64),
                has_nfo: if sidecars.has_nfo { 1 } else { 0 },
                has_poster: if sidecars.has_poster { 1 } else { 0 },
                has_backdrop: if sidecars.has_backdrop { 1 } else { 0 },
                file_size: Some(file_size as i64),
                subtitle_count: sidecars.subtitle_count as i32,
                version_label,
                scan_status: scan_status.to_string(),
                created_at: String::new(),
                updated_at: String::new(),
            };

            if let Ok(conn) = conn_mutex.lock() {
                if insert_movie(&conn, &movie).is_ok() {
                    movies_found += 1;
                }
            }
        }
    }

    if let Ok(conn) = conn_mutex.lock() {
        let _ = update_library_root_scan_time(&conn, root_id);
    }

    ScanResult {
        root_id,
        movies_found,
        shows_found: 0,
        episodes_found: 0,
        duration_ms: start_time.elapsed().as_millis() as u64,
    }
}

/// Scans a TV show library root.
/// Expects each subfolder to represent a TV series, containing Season XX folders or episode files.
pub fn scan_tvshow_root(
    root_path: &str,
    root_id: i64,
    conn_mutex: &Mutex<Connection>,
    app_handle: Option<&tauri::AppHandle>,
) -> ScanResult {
    let start_time = Instant::now();
    let root = Path::new(root_path);
    let mut shows_found = 0;
    let mut episodes_found = 0;

    let entries: Vec<PathBuf> = match std::fs::read_dir(root) {
        Ok(read_dir) => read_dir
            .flatten()
            .map(|e| e.path())
            .filter(|p| !is_hidden_or_extra(p))
            .collect(),
        Err(_) => {
            return ScanResult {
                root_id,
                movies_found: 0,
                shows_found: 0,
                episodes_found: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
            }
        }
    };

    let total_dirs = entries.len();

    for (index, show_dir) in entries.iter().enumerate() {
        if !show_dir.is_dir() {
            continue;
        }

        let show_dir_str = show_dir.to_string_lossy().to_string();
        let folder_name = show_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if let Some(handle) = app_handle {
            let _ = handle.emit(
                "scan:progress",
                ScanProgress {
                    scanned: index + 1,
                    total_dirs,
                    current_path: show_dir_str.clone(),
                },
            );
        }

        let parsed_show = parse_filename(folder_name);
        let show_title = parsed_show
            .title
            .unwrap_or_else(|| folder_name.to_string());

        let sidecars = detect_sidecars(show_dir, folder_name);

        let scan_status = if parsed_show.tvdb_id.is_some() || parsed_show.tmdb_id.is_some() || parsed_show.imdb_id.is_some() {
            "matched"
        } else {
            "unmatched"
        };

        let tv_show = TvShow {
            id: 0,
            library_root_id: root_id,
            folder_path: show_dir_str.clone(),
            title: show_title,
            sort_title: None,
            year: parsed_show.year.map(|y| y as i32),
            tvdb_id: parsed_show.tvdb_id.map(|id| id as i64),
            tmdb_id: parsed_show.tmdb_id.map(|id| id as i64),
            imdb_id: parsed_show.imdb_id,
            status: None,
            has_nfo: if sidecars.has_nfo { 1 } else { 0 },
            has_poster: if sidecars.has_poster { 1 } else { 0 },
            has_backdrop: if sidecars.has_backdrop { 1 } else { 0 },
            season_count: 0,
            episode_count: 0,
            scan_status: scan_status.to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        };

        let show_id = match conn_mutex.lock() {
            Ok(conn) => match insert_tv_show(&conn, &tv_show) {
                Ok(id) => {
                    shows_found += 1;
                    id
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let mut show_episodes_found = 0;
        let mut show_seasons_set = std::collections::HashSet::new();

        // Scan for episode files recursively inside this show directory (handles unorganized trees without Season XX folders)
        for entry in WalkDir::new(show_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_hidden_or_extra(e.path()))
            .flatten()
        {
            let path = entry.path();
            if path.is_file() && is_video_file(path) {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                let parsed_ep = parse_filename(filename);

                let parent_dir = path.parent().unwrap_or(show_dir);
                let parent_name = parent_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Resolve season: from filename parser -> from parent folder name -> default 1
                let season = parsed_ep.season.map(|s| s as i32).unwrap_or_else(|| {
                    let parent_lower = parent_name.to_lowercase();
                    if parent_lower.contains("special") {
                        0
                    } else if let Some(cap) = SEASON_FOLDER_REGEX.captures(parent_name) {
                        cap[1].parse::<i32>().unwrap_or(1)
                    } else {
                        1
                    }
                });

                // Resolve episode: from filename parser -> leading digits in filename -> default 1
                let episode_num = parsed_ep.episode.map(|e| e as i32).unwrap_or_else(|| {
                    if let Some(cap) = LEADING_EP_NUM_REGEX.captures(filename) {
                        cap[1].parse::<i32>().unwrap_or(1)
                    } else {
                        1
                    }
                });

                let ep_sidecars = detect_sidecars(parent_dir, stem);
                let file_size = std::fs::metadata(path).map(|m| m.len() as i64).ok();

                let ep_status = if parsed_ep.season.is_some() && parsed_ep.episode.is_some() {
                    "matched"
                } else {
                    "unmatched"
                };

                let episode = Episode {
                    id: 0,
                    tv_show_id: show_id,
                    file_path: path.to_string_lossy().to_string(),
                    title: parsed_ep.title,
                    season,
                    episode: episode_num,
                    episode_end: parsed_ep.episode_end.map(|e| e as i32),
                    resolution: parsed_ep.resolution,
                    imdb_id: parsed_ep.imdb_id,
                    tvdb_id: parsed_ep.tvdb_id.map(|id| id as i64),
                    has_nfo: if ep_sidecars.has_nfo { 1 } else { 0 },
                    file_size,
                    subtitle_count: ep_sidecars.subtitle_count as i32,
                    scan_status: ep_status.to_string(),
                    created_at: String::new(),
                    updated_at: String::new(),
                };

                if let Ok(conn) = conn_mutex.lock() {
                    if insert_episode(&conn, &episode).is_ok() {
                        episodes_found += 1;
                        show_episodes_found += 1;
                        show_seasons_set.insert(season);
                    }
                }
            }
        }

        // Update show season and episode counts in DB
        if let Ok(conn) = conn_mutex.lock() {
            let _ = conn.execute(
                "UPDATE tv_shows SET season_count = ?1, episode_count = ?2, updated_at = datetime('now') WHERE id = ?3",
                params![show_seasons_set.len() as i32, show_episodes_found, show_id],
            );
        }
    }

    if let Ok(conn) = conn_mutex.lock() {
        let _ = update_library_root_scan_time(&conn, root_id);
    }

    ScanResult {
        root_id,
        movies_found: 0,
        shows_found,
        episodes_found,
        duration_ms: start_time.elapsed().as_millis() as u64,
    }
}
