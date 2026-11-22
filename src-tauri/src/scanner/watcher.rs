use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::db::models::LibraryRoot;
use crate::renamer::executor::IS_RENAMING;
use crate::scanner::walker::{is_video_file, scan_movie_root, scan_tvshow_root, EXTRAS_FOLDERS};

const IGNORED_EXTENSIONS: &[&str] = &[
    "tmp", "crdownload", "part", "aria2", "torrent", "bak", "sample", "log", "DS_Store",
];

const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "sub", "ass", "vtt", "idx"];

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "tbn"];

/// Checks if a file/directory path is relevant for media scanning.
pub fn is_relevant_path(path: &Path) -> bool {
    // Check all components for hidden folders (.git, .thumbnails, etc.) or archive folders (_duplicates_archive)
    for comp in path.components() {
        if let Some(s) = comp.as_os_str().to_str() {
            if (s.starts_with('.') && s != "." && s != "..") || s.starts_with('_') {
                return false;
            }
            if EXTRAS_FOLDERS.contains(&s.to_lowercase().as_str()) {
                return false;
            }
        }
    }

    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        let fn_lower = file_name.to_lowercase();
        if fn_lower.contains("sample") {
            return false;
        }
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if IGNORED_EXTENSIONS.contains(&ext_lower.as_str()) {
            return false;
        }

        if is_video_file(path) {
            return true;
        }

        if SUBTITLE_EXTENSIONS.contains(&ext_lower.as_str()) {
            return true;
        }

        if ext_lower == "nfo" {
            return true;
        }

        if IMAGE_EXTENSIONS.contains(&ext_lower.as_str()) {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let stem_lower = stem.to_lowercase();
                if stem_lower.starts_with("poster")
                    || stem_lower.starts_with("backdrop")
                    || stem_lower.starts_with("fanart")
                    || stem_lower.starts_with("banner")
                    || stem_lower.starts_with("folder")
                    || stem_lower.starts_with("cover")
                    || stem_lower.starts_with("season")
                {
                    return true;
                }
            }
        }
        false
    } else {
        // No extension: directory (e.g. movie folder, TV show folder, season folder)
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStatus {
    pub enabled: bool,
    pub watched_roots_count: usize,
    pub watched_paths: Vec<String>,
    pub last_event_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherChangeEvent {
    pub root_id: i64,
    pub media_type: String,
    pub paths: Vec<String>,
    pub timestamp: String,
}

pub struct LibraryWatcher {
    debouncer: Option<Debouncer<RecommendedWatcher>>,
    watched_roots: Vec<LibraryRoot>,
    enabled: bool,
    last_event_at: Option<String>,
    app_handle: Option<tauri::AppHandle>,
    db_conn: Option<Arc<Mutex<Connection>>>,
}

impl LibraryWatcher {
    pub fn new() -> Self {
        Self {
            debouncer: None,
            watched_roots: Vec::new(),
            enabled: true,
            last_event_at: None,
            app_handle: None,
            db_conn: None,
        }
    }

    pub fn start(
        &mut self,
        roots: Vec<LibraryRoot>,
        app_handle: Option<tauri::AppHandle>,
        db_conn: Arc<Mutex<Connection>>,
    ) -> Result<(), String> {
        self.stop();

        self.watched_roots = roots.into_iter().filter(|r| r.enabled == 1).collect();
        self.app_handle = app_handle.clone();
        self.db_conn = Some(db_conn.clone());

        if !self.enabled || self.watched_roots.is_empty() {
            return Ok(());
        }

        let roots_clone = self.watched_roots.clone();
        let app_handle_clone = app_handle.clone();
        let db_conn_clone = db_conn.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(1500),
            move |res: Result<Vec<DebouncedEvent>, _>| {
                if IS_RENAMING.load(Ordering::SeqCst) {
                    log::debug!("Ignoring watcher event during active rename batch");
                    return;
                }

                match res {
                    Ok(events) => {
                        let relevant_paths: Vec<PathBuf> = events
                            .into_iter()
                            .map(|e| e.path)
                            .filter(|p| is_relevant_path(p))
                            .collect();

                        if relevant_paths.is_empty() {
                            return;
                        }

                        log::info!(
                            "Watcher detected {} relevant filesystem changes",
                            relevant_paths.len()
                        );

                        // Group paths by library root
                        let mut affected_roots: std::collections::HashMap<i64, (LibraryRoot, Vec<String>)> =
                            std::collections::HashMap::new();

                        for path in relevant_paths {
                            for root in &roots_clone {
                                if path.starts_with(&root.path) {
                                    let entry = affected_roots
                                        .entry(root.id)
                                        .or_insert_with(|| (root.clone(), Vec::new()));
                                    entry.1.push(path.to_string_lossy().to_string());
                                    break;
                                }
                            }
                        }

                        let now = chrono::Local::now().to_rfc3339();

                        for (_root_id, (root, paths)) in affected_roots {
                            log::info!(
                                "Auto-rescanning library root '{}' ({} paths changed)",
                                root.path,
                                paths.len()
                            );

                            let scan_result = if root.media_type == "movie" {
                                scan_movie_root(
                                    &root.path,
                                    root.id,
                                    &db_conn_clone,
                                    app_handle_clone.as_ref(),
                                )
                            } else {
                                scan_tvshow_root(
                                    &root.path,
                                    root.id,
                                    &db_conn_clone,
                                    app_handle_clone.as_ref(),
                                )
                            };

                            if let Some(ref handle) = app_handle_clone {
                                let _ = handle.emit(
                                    "watcher:change",
                                    WatcherChangeEvent {
                                        root_id: root.id,
                                        media_type: root.media_type.clone(),
                                        paths,
                                        timestamp: now.clone(),
                                    },
                                );

                                let _ = handle.emit("scan:complete", scan_result);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Watcher error: {:?}", e);
                    }
                }
            },
        )
        .map_err(|e| e.to_string())?;

        for root in &self.watched_roots {
            let path = Path::new(&root.path);
            if path.exists() {
                if let Err(e) = debouncer.watcher().watch(path, RecursiveMode::Recursive) {
                    log::warn!("Failed to watch library root {}: {}", root.path, e);
                } else {
                    log::info!("Watcher active on: {}", root.path);
                }
            }
        }

        self.debouncer = Some(debouncer);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut debouncer) = self.debouncer.take() {
            for root in &self.watched_roots {
                let _ = debouncer.watcher().unwatch(Path::new(&root.path));
            }
        }
    }

    pub fn reload(
        &mut self,
        roots: Vec<LibraryRoot>,
        app_handle: Option<tauri::AppHandle>,
        db_conn: Arc<Mutex<Connection>>,
    ) -> Result<(), String> {
        self.start(roots, app_handle, db_conn)
    }

    pub fn set_enabled(
        &mut self,
        enabled: bool,
        roots: Vec<LibraryRoot>,
        app_handle: Option<tauri::AppHandle>,
        db_conn: Arc<Mutex<Connection>>,
    ) -> Result<(), String> {
        self.enabled = enabled;
        if enabled {
            self.start(roots, app_handle, db_conn)
        } else {
            self.stop();
            Ok(())
        }
    }

    pub fn get_status(&self) -> WatcherStatus {
        let watched_paths = if self.enabled && self.debouncer.is_some() {
            self.watched_roots.iter().map(|r| r.path.clone()).collect()
        } else {
            Vec::new()
        };

        WatcherStatus {
            enabled: self.enabled,
            watched_roots_count: watched_paths.len(),
            watched_paths,
            last_event_at: self.last_event_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_relevant_path_video() {
        assert!(is_relevant_path(Path::new("/media/movies/Alien (1979)/Alien.mkv")));
        assert!(is_relevant_path(Path::new("/media/shows/Show/S01E01.mp4")));
    }

    #[test]
    fn test_is_relevant_path_sidecars() {
        assert!(is_relevant_path(Path::new("/media/movies/Alien (1979)/Alien.en.srt")));
        assert!(is_relevant_path(Path::new("/media/movies/Alien (1979)/movie.nfo")));
        assert!(is_relevant_path(Path::new("/media/movies/Alien (1979)/poster.jpg")));
        assert!(is_relevant_path(Path::new("/media/movies/Alien (1979)/backdrop.png")));
    }

    #[test]
    fn test_is_relevant_path_ignored() {
        assert!(!is_relevant_path(Path::new("/media/movies/.git/HEAD")));
        assert!(!is_relevant_path(Path::new("/media/movies/.DS_Store")));
        assert!(!is_relevant_path(Path::new("/media/movies/Alien (1979)/movie.mkv.tmp")));
        assert!(!is_relevant_path(Path::new("/media/movies/Alien (1979)/movie.mkv.crdownload")));
        assert!(!is_relevant_path(Path::new("/media/movies/Alien (1979)/trailers/trailer.mkv")));
    }
}
