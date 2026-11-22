use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::{Episode, Movie, TvShow};
use crate::db::queries::{get_episodes_for_show, get_movie_by_id, get_movies, get_tv_show_by_id, get_tv_shows};
use crate::naming::jellyfin::{
    format_episode_filename, format_movie_filename, format_movie_folder, format_provider_tag,
    format_season_folder, format_tv_show_folder,
};
use crate::renamer::sidecars::{
    find_episode_sidecars, find_movie_sidecars, find_series_sidecars, SidecarKind,
};
use crate::scanner::walker::VIDEO_EXTENSIONS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    WillRename,
    Skipped,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenameItemType {
    MovieFolder,
    MovieVideo,
    TvShowFolder,
    SeasonFolder,
    EpisodeVideo,
    Subtitle,
    Nfo,
    Artwork,
    Extra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameAction {
    pub id: String,
    pub media_type: String, // "movie" | "tvshow"
    pub media_id: i64,
    pub item_type: RenameItemType,
    pub old_path: String,
    pub new_path: String,
    pub status: ActionStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamePlan {
    pub batch_id: String,
    pub actions: Vec<RenameAction>,
    pub total_count: usize,
    pub will_rename_count: usize,
    pub skipped_count: usize,
    pub conflict_count: usize,
}

impl RenamePlan {
    pub fn new(batch_id: String, mut actions: Vec<RenameAction>) -> Self {
        validate_and_tag_actions(&mut actions);

        let total_count = actions.len();
        let will_rename_count = actions.iter().filter(|a| a.status == ActionStatus::WillRename).count();
        let skipped_count = actions.iter().filter(|a| a.status == ActionStatus::Skipped).count();
        let conflict_count = actions.iter().filter(|a| a.status == ActionStatus::Conflict).count();

        Self {
            batch_id,
            actions,
            total_count,
            will_rename_count,
            skipped_count,
            conflict_count,
        }
    }
}

/// Validates actions in a plan, marking duplicate destinations or existing file conflicts.
fn validate_and_tag_actions(actions: &mut [RenameAction]) {
    let mut dest_counts: HashMap<String, usize> = HashMap::new();
    for a in actions.iter() {
        if a.old_path != a.new_path {
            *dest_counts.entry(a.new_path.clone()).or_insert(0) += 1;
        }
    }

    for action in actions.iter_mut() {
        if action.old_path == action.new_path {
            action.status = ActionStatus::Skipped;
            action.reason = Some("Path already complies with Jellyfin standards".into());
            continue;
        }

        // Check if multiple actions map to the same destination
        if let Some(&count) = dest_counts.get(&action.new_path) {
            if count > 1 {
                action.status = ActionStatus::Conflict;
                action.reason = Some("Collision: multiple items target this same path".into());
                continue;
            }
        }

        // Check if destination exists on disk (and is not identical case-insensitive)
        let new_p = Path::new(&action.new_path);
        let old_p = Path::new(&action.old_path);
        if new_p.exists() {
            let is_same_file = match (new_p.canonicalize(), old_p.canonicalize()) {
                (Ok(c1), Ok(c2)) => c1 == c2,
                _ => false,
            };

            if !is_same_file {
                // If it's a directory and old_path is being merged into it, or existing file
                if new_p.is_file() {
                    action.status = ActionStatus::Conflict;
                    action.reason = Some("Destination file already exists on disk".into());
                    continue;
                }
            }
        }

        action.status = ActionStatus::WillRename;
        action.reason = None;
    }
}

/// Generates a rename plan for a single movie and all its sidecars.
pub fn plan_movie_rename(conn: &Connection, movie_id: i64) -> Result<RenamePlan, String> {
    let movie = get_movie_by_id(conn, movie_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Movie id {} not found", movie_id))?;

    let batch_id = Uuid::new_v4().to_string();
    let mut actions = Vec::new();
    collect_movie_actions(&movie, &mut actions);

    Ok(RenamePlan::new(batch_id, actions))
}

/// Generates a rename plan for a TV show, all its seasons, episodes, and sidecars.
pub fn plan_tv_show_rename(conn: &Connection, tv_show_id: i64) -> Result<RenamePlan, String> {
    let show = get_tv_show_by_id(conn, tv_show_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("TV Show id {} not found", tv_show_id))?;

    let episodes = get_episodes_for_show(conn, tv_show_id).map_err(|e| e.to_string())?;

    let batch_id = Uuid::new_v4().to_string();
    let mut actions = Vec::new();
    collect_tv_show_actions(&show, &episodes, &mut actions);

    Ok(RenamePlan::new(batch_id, actions))
}

/// Generates a combined rename plan for a list of movies and TV shows.
pub fn plan_batch_rename(
    conn: &Connection,
    movie_ids: &[i64],
    tv_show_ids: &[i64],
) -> Result<RenamePlan, String> {
    let batch_id = Uuid::new_v4().to_string();
    let mut actions = Vec::new();

    // If empty lists provided, plan for all movies and tv shows
    let target_movies: Vec<Movie> = if movie_ids.is_empty() && tv_show_ids.is_empty() {
        get_movies(conn, None).map_err(|e| e.to_string())?
    } else {
        let mut list = Vec::new();
        for &id in movie_ids {
            if let Ok(Some(m)) = get_movie_by_id(conn, id) {
                list.push(m);
            }
        }
        list
    };

    let target_shows: Vec<(TvShow, Vec<Episode>)> = if movie_ids.is_empty() && tv_show_ids.is_empty() {
        let shows = get_tv_shows(conn, None).map_err(|e| e.to_string())?;
        let mut list = Vec::new();
        for s in shows {
            let eps = get_episodes_for_show(conn, s.id).unwrap_or_default();
            list.push((s, eps));
        }
        list
    } else {
        let mut list = Vec::new();
        for &id in tv_show_ids {
            if let Ok(Some(s)) = get_tv_show_by_id(conn, id) {
                let eps = get_episodes_for_show(conn, s.id).unwrap_or_default();
                list.push((s, eps));
            }
        }
        list
    };

    for m in &target_movies {
        collect_movie_actions(m, &mut actions);
    }

    for (s, eps) in &target_shows {
        collect_tv_show_actions(s, eps, &mut actions);
    }

    Ok(RenamePlan::new(batch_id, actions))
}

fn collect_movie_actions(movie: &Movie, actions: &mut Vec<RenameAction>) {
    let old_folder = PathBuf::from(&movie.folder_path);
    let library_parent = match old_folder.parent() {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("."),
    };

    let provider_tag = format_provider_tag(
        movie.imdb_id.as_deref(),
        movie.tmdb_id.map(|v| v as u32),
        None,
        true,
    );

    let target_folder_name = format_movie_folder(
        &movie.title,
        movie.year.map(|y| y as u16),
        provider_tag.as_deref(),
    );
    let target_folder_path = library_parent.join(&target_folder_name);

    // 1. Movie Folder Action
    actions.push(RenameAction {
        id: Uuid::new_v4().to_string(),
        media_type: "movie".into(),
        media_id: movie.id,
        item_type: RenameItemType::MovieFolder,
        old_path: old_folder.to_string_lossy().to_string(),
        new_path: target_folder_path.to_string_lossy().to_string(),
        status: ActionStatus::WillRename,
        reason: None,
    });

    // Detect all video files inside old_folder to see if it is multi-version
    let mut video_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&old_folder) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                        video_files.push(p);
                    }
                }
            }
        }
    }

    let is_multi_version = video_files.len() > 1;

    // If video_files is empty (e.g. mock missing on disk), use movie.file_path
    if video_files.is_empty() {
        video_files.push(PathBuf::from(&movie.file_path));
    }

    for vid in video_files {
        let ext = vid.extension().and_then(|e| e.to_str()).unwrap_or("mkv");
        let old_stem = vid.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // Multi-version label handling
        let version_label = if is_multi_version {
            // Check if movie struct has version_label, or parse from resolution/stem
            movie.version_label.as_deref().or_else(|| {
                if old_stem.contains("1080p") {
                    Some("1080p")
                } else if old_stem.contains("2160p") || old_stem.contains("4k") {
                    Some("2160p")
                } else if old_stem.contains("720p") {
                    Some("720p")
                } else {
                    None
                }
            })
        } else {
            None
        };

        let target_video_name = format_movie_filename(&target_folder_name, version_label, ext);
        let target_video_path = target_folder_path.join(&target_video_name);

        actions.push(RenameAction {
            id: Uuid::new_v4().to_string(),
            media_type: "movie".into(),
            media_id: movie.id,
            item_type: RenameItemType::MovieVideo,
            old_path: vid.to_string_lossy().to_string(),
            new_path: target_video_path.to_string_lossy().to_string(),
            status: ActionStatus::WillRename,
            reason: None,
        });

        // 3. Sidecars for this video
        let new_stem = target_video_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&target_video_name);

        let sidecars = find_movie_sidecars(&old_folder, old_stem, new_stem, is_multi_version);

        for sc in sidecars {
            let item_type = match sc.kind {
                SidecarKind::Subtitle => RenameItemType::Subtitle,
                SidecarKind::Nfo => RenameItemType::Nfo,
                SidecarKind::Artwork => RenameItemType::Artwork,
                SidecarKind::ExtraFolder => RenameItemType::Extra,
            };

            let target_sc_path = target_folder_path.join(&sc.new_filename);

            actions.push(RenameAction {
                id: Uuid::new_v4().to_string(),
                media_type: "movie".into(),
                media_id: movie.id,
                item_type,
                old_path: sc.old_path.to_string_lossy().to_string(),
                new_path: target_sc_path.to_string_lossy().to_string(),
                status: ActionStatus::WillRename,
                reason: None,
            });
        }
    }
}

fn collect_tv_show_actions(show: &TvShow, episodes: &[Episode], actions: &mut Vec<RenameAction>) {
    let old_show_folder = PathBuf::from(&show.folder_path);
    let library_parent = match old_show_folder.parent() {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("."),
    };

    let tag = format_provider_tag(
        show.imdb_id.as_deref(),
        show.tmdb_id.map(|v| v as u32),
        show.tvdb_id.map(|v| v as u32),
        false,
    );

    let target_show_name = format_tv_show_folder(
        &show.title,
        show.year.map(|y| y as u16),
        tag.as_deref(),
    );
    let target_show_path = library_parent.join(&target_show_name);

    // 1. Series Folder Action
    actions.push(RenameAction {
        id: Uuid::new_v4().to_string(),
        media_type: "tvshow".into(),
        media_id: show.id,
        item_type: RenameItemType::TvShowFolder,
        old_path: old_show_folder.to_string_lossy().to_string(),
        new_path: target_show_path.to_string_lossy().to_string(),
        status: ActionStatus::WillRename,
        reason: None,
    });

    // 2. Series Root Sidecars (tvshow.nfo, poster.jpg, season*-poster.jpg)
    let series_sidecars = find_series_sidecars(&old_show_folder);
    for sc in series_sidecars {
        let item_type = match sc.kind {
            SidecarKind::Nfo => RenameItemType::Nfo,
            _ => RenameItemType::Artwork,
        };

        let target_sc_path = target_show_path.join(&sc.new_filename);
        actions.push(RenameAction {
            id: Uuid::new_v4().to_string(),
            media_type: "tvshow".into(),
            media_id: show.id,
            item_type,
            old_path: sc.old_path.to_string_lossy().to_string(),
            new_path: target_sc_path.to_string_lossy().to_string(),
            status: ActionStatus::WillRename,
            reason: None,
        });
    }

    // 3. Episodes and their Season Folders
    let mut planned_seasons = HashSet::new();

    for ep in episodes {
        let old_ep_path = PathBuf::from(&ep.file_path);
        let season_folder_name = format_season_folder(ep.season as u16);
        let target_season_path = target_show_path.join(&season_folder_name);

        // Record Season Folder if not already recorded
        if !planned_seasons.contains(&ep.season) {
            planned_seasons.insert(ep.season);
            if let Some(old_parent) = old_ep_path.parent() {
                actions.push(RenameAction {
                    id: Uuid::new_v4().to_string(),
                    media_type: "tvshow".into(),
                    media_id: show.id,
                    item_type: RenameItemType::SeasonFolder,
                    old_path: old_parent.to_string_lossy().to_string(),
                    new_path: target_season_path.to_string_lossy().to_string(),
                    status: ActionStatus::WillRename,
                    reason: None,
                });
            }
        }

        let ext = old_ep_path.extension().and_then(|e| e.to_str()).unwrap_or("mkv");
        let target_ep_name = format_episode_filename(
            &show.title,
            ep.season as u16,
            ep.episode as u16,
            ep.episode_end.map(|e| e as u16),
            ep.title.as_deref(),
            ext,
        );

        let target_ep_path = target_season_path.join(&target_ep_name);

        actions.push(RenameAction {
            id: Uuid::new_v4().to_string(),
            media_type: "tvshow".into(),
            media_id: show.id,
            item_type: RenameItemType::EpisodeVideo,
            old_path: old_ep_path.to_string_lossy().to_string(),
            new_path: target_ep_path.to_string_lossy().to_string(),
            status: ActionStatus::WillRename,
            reason: None,
        });

        // 4. Episode Sidecars (subtitles & episode NFOs)
        if let Some(old_parent) = old_ep_path.parent() {
            let old_stem = old_ep_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let new_stem = target_ep_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            let ep_sidecars = find_episode_sidecars(old_parent, old_stem, new_stem);
            for sc in ep_sidecars {
                let item_type = match sc.kind {
                    SidecarKind::Subtitle => RenameItemType::Subtitle,
                    _ => RenameItemType::Nfo,
                };

                let target_sc_path = target_season_path.join(&sc.new_filename);
                actions.push(RenameAction {
                    id: Uuid::new_v4().to_string(),
                    media_type: "tvshow".into(),
                    media_id: show.id,
                    item_type,
                    old_path: sc.old_path.to_string_lossy().to_string(),
                    new_path: target_sc_path.to_string_lossy().to_string(),
                    status: ActionStatus::WillRename,
                    reason: None,
                });
            }
        }
    }
}

/// Plans resolving duplicates by archiving obsolete duplicates into a `_duplicates_archive/` subfolder.
/// This preserves the files safely on disk and logs every move to `rename_log` for 1-click Undo.
pub fn plan_resolve_duplicates(
    conn: &Connection,
    keep_file_path: &str,
    obsolete_file_paths: &[String],
    media_type: &str, // "movie" | "episode"
) -> Result<RenamePlan, String> {
    let mut actions = Vec::new();
    let batch_id = Uuid::new_v4().to_string();
    let keep_name = Path::new(keep_file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(keep_file_path);

    for obsolete_path_str in obsolete_file_paths {
        let old_p = Path::new(obsolete_path_str);
        if !old_p.exists() {
            actions.push(RenameAction {
                id: Uuid::new_v4().to_string(),
                media_type: media_type.to_string(),
                media_id: 0,
                item_type: if media_type == "movie" {
                    RenameItemType::MovieVideo
                } else {
                    RenameItemType::EpisodeVideo
                },
                old_path: obsolete_path_str.clone(),
                new_path: obsolete_path_str.clone(),
                status: ActionStatus::Skipped,
                reason: Some("Source file no longer exists".into()),
            });
            continue;
        }

        // Look up media ID from database
        let media_id: i64 = if media_type == "movie" {
            conn.query_row(
                "SELECT id FROM movies WHERE file_path = ?1",
                params![obsolete_path_str],
                |r| r.get(0),
            )
            .unwrap_or(0)
        } else {
            conn.query_row(
                "SELECT id FROM episodes WHERE file_path = ?1",
                params![obsolete_path_str],
                |r| r.get(0),
            )
            .unwrap_or(0)
        };

        let parent = old_p.parent().unwrap_or(Path::new("."));
        let archive_dir = parent.join("_duplicates_archive");
        let file_name = match old_p.file_name() {
            Some(n) => n,
            None => continue,
        };
        let new_p = archive_dir.join(file_name);

        let status = if new_p.exists() {
            ActionStatus::Conflict
        } else {
            ActionStatus::WillRename
        };

        let reason = if status == ActionStatus::Conflict {
            Some("Destination file already exists in _duplicates_archive".into())
        } else {
            Some(format!(
                "Archive duplicate to _duplicates_archive (Keeping: {})",
                keep_name
            ))
        };

        actions.push(RenameAction {
            id: Uuid::new_v4().to_string(),
            media_type: media_type.to_string(),
            media_id,
            item_type: if media_type == "movie" {
                RenameItemType::MovieVideo
            } else {
                RenameItemType::EpisodeVideo
            },
            old_path: old_p.to_string_lossy().to_string(),
            new_path: new_p.to_string_lossy().to_string(),
            status,
            reason,
        });

        // Also archive any matching sidecars
        if let Some(stem) = old_p.file_stem().and_then(|s| s.to_str()) {
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let ep_path = entry.path();
                    if ep_path.is_file() && ep_path != old_p {
                        let ep_name = ep_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        if ep_name.starts_with(stem) {
                            let ext = ep_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if matches!(ext.as_str(), "srt" | "sub" | "vtt" | "ass" | "nfo") {
                                let sc_new_p = archive_dir.join(ep_name);
                                let sc_status = if sc_new_p.exists() {
                                    ActionStatus::Conflict
                                } else {
                                    ActionStatus::WillRename
                                };
                                actions.push(RenameAction {
                                    id: Uuid::new_v4().to_string(),
                                    media_type: media_type.to_string(),
                                    media_id,
                                    item_type: if ext == "nfo" {
                                        RenameItemType::Nfo
                                    } else {
                                        RenameItemType::Subtitle
                                    },
                                    old_path: ep_path.to_string_lossy().to_string(),
                                    new_path: sc_new_p.to_string_lossy().to_string(),
                                    status: sc_status,
                                    reason: Some("Archive sidecar alongside duplicate video".into()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let total_count = actions.len();
    let will_rename_count = actions.iter().filter(|a| a.status == ActionStatus::WillRename).count();
    let skipped_count = actions.iter().filter(|a| a.status == ActionStatus::Skipped).count();
    let conflict_count = actions.iter().filter(|a| a.status == ActionStatus::Conflict).count();

    Ok(RenamePlan {
        batch_id,
        actions,
        total_count,
        will_rename_count,
        skipped_count,
        conflict_count,
    })
}
