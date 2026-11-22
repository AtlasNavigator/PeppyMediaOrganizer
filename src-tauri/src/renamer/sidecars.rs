use std::path::{Path, PathBuf};

pub const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "sub", "idx", "ass", "ssa", "vtt", "sup"];
pub const ARTWORK_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "tbn"];
pub const STANDARD_ARTWORK_NAMES: &[&str] = &[
    "poster", "backdrop", "fanart", "banner", "logo", "clearart", "thumb", "disc"
];
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarKind {
    Subtitle,
    Nfo,
    Artwork,
    ExtraFolder,
}

#[derive(Debug, Clone)]
pub struct DiscoveredSidecar {
    pub old_path: PathBuf,
    pub new_filename: String,
    pub kind: SidecarKind,
}

/// Finds all sidecars for a movie in `movie_folder`.
/// When `old_video_stem` is renamed to `new_video_stem`, generates the corresponding new filenames.
pub fn find_movie_sidecars(
    movie_folder: &Path,
    old_video_stem: &str,
    new_video_stem: &str,
    is_multi_version: bool,
) -> Vec<DiscoveredSidecar> {
    let mut sidecars = Vec::new();

    let entries = match std::fs::read_dir(movie_folder) {
        Ok(read_dir) => read_dir.flatten().collect::<Vec<_>>(),
        Err(_) => return sidecars,
    };

    let old_stem_lower = old_video_stem.to_lowercase();

    for entry in entries {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if path.is_dir() {
            let lower_dir = file_name.to_lowercase();
            if EXTRAS_FOLDERS.contains(&lower_dir.as_str()) {
                sidecars.push(DiscoveredSidecar {
                    old_path: path.clone(),
                    new_filename: file_name.to_string(),
                    kind: SidecarKind::ExtraFolder,
                });
            }
            continue;
        }

        let file_lower = file_name.to_lowercase();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Subtitles
        if SUBTITLE_EXTENSIONS.contains(&ext.as_str()) {
            if file_lower.starts_with(&old_stem_lower) {
                // e.g. "Movie (1982).en.forced.srt" -> suffix is ".en.forced.srt"
                let suffix = &file_name[old_video_stem.len()..];
                let new_name = format!("{}{}", new_video_stem, suffix);
                sidecars.push(DiscoveredSidecar {
                    old_path: path.clone(),
                    new_filename: new_name,
                    kind: SidecarKind::Subtitle,
                });
            } else if file_lower.starts_with("en.")
                || file_lower.starts_with("eng.")
                || file_lower.starts_with("es.")
                || file_lower.starts_with("spa.")
                || file_lower.starts_with("fr.")
                || file_lower.starts_with("fre.")
                || file_lower.starts_with("de.")
                || file_lower.starts_with("ger.")
                || file_lower.starts_with("ja.")
                || file_lower.starts_with("jpn.")
                || file_lower == "subtitle.srt"
                || file_lower == "subtitles.srt"
            {
                let new_name = format!("{}.{}", new_video_stem, file_name);
                sidecars.push(DiscoveredSidecar {
                    old_path: path.clone(),
                    new_filename: new_name,
                    kind: SidecarKind::Subtitle,
                });
            }
            continue;
        }

        // 2. NFO Files
        if ext == "nfo" {
            let new_nfo_name = if is_multi_version {
                format!("{}.nfo", new_video_stem)
            } else {
                "movie.nfo".to_string()
            };

            sidecars.push(DiscoveredSidecar {
                old_path: path.clone(),
                new_filename: new_nfo_name,
                kind: SidecarKind::Nfo,
            });
            continue;
        }

        // 3. Artwork
        if ARTWORK_EXTENSIONS.contains(&ext.as_str()) {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            let target_name = if STANDARD_ARTWORK_NAMES.contains(&stem.as_str()) {
                file_name.to_string()
            } else if stem.ends_with("-poster") || stem.ends_with(".poster") {
                format!("poster.{}", ext)
            } else if stem.ends_with("-fanart") || stem.ends_with("-backdrop") || stem.ends_with(".backdrop") {
                format!("backdrop.{}", ext)
            } else if stem.ends_with("-banner") {
                format!("banner.{}", ext)
            } else if stem.ends_with("-logo") {
                format!("logo.{}", ext)
            } else {
                file_name.to_string()
            };

            sidecars.push(DiscoveredSidecar {
                old_path: path.clone(),
                new_filename: target_name,
                kind: SidecarKind::Artwork,
            });
        }
    }

    sidecars
}

/// Finds sidecars for a TV episode in its season folder.
pub fn find_episode_sidecars(
    season_folder: &Path,
    old_ep_stem: &str,
    new_ep_stem: &str,
) -> Vec<DiscoveredSidecar> {
    let mut sidecars = Vec::new();

    let entries = match std::fs::read_dir(season_folder) {
        Ok(read_dir) => read_dir.flatten().collect::<Vec<_>>(),
        Err(_) => return sidecars,
    };

    let old_stem_lower = old_ep_stem.to_lowercase();

    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        let file_lower = file_name.to_lowercase();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Episode Subtitles
        if SUBTITLE_EXTENSIONS.contains(&ext.as_str()) && file_lower.starts_with(&old_stem_lower) {
            let suffix = &file_name[old_ep_stem.len()..];
            let new_name = format!("{}{}", new_ep_stem, suffix);
            sidecars.push(DiscoveredSidecar {
                old_path: path.clone(),
                new_filename: new_name,
                kind: SidecarKind::Subtitle,
            });
            continue;
        }

        // 2. Episode NFO
        if ext == "nfo" && (file_lower.starts_with(&old_stem_lower) || file_lower == format!("{}.nfo", old_stem_lower)) {
            let new_name = format!("{}.nfo", new_ep_stem);
            sidecars.push(DiscoveredSidecar {
                old_path: path.clone(),
                new_filename: new_name,
                kind: SidecarKind::Nfo,
            });
        }
    }

    sidecars
}

/// Finds series-level artwork and `tvshow.nfo` in the TV show folder.
pub fn find_series_sidecars(show_folder: &Path) -> Vec<DiscoveredSidecar> {
    let mut sidecars = Vec::new();

    let entries = match std::fs::read_dir(show_folder) {
        Ok(read_dir) => read_dir.flatten().collect::<Vec<_>>(),
        Err(_) => return sidecars,
    };

    for entry in entries {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if path.is_file() {
            let file_lower = file_name.to_lowercase();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            if file_lower == "tvshow.nfo" {
                sidecars.push(DiscoveredSidecar {
                    old_path: path.clone(),
                    new_filename: "tvshow.nfo".to_string(),
                    kind: SidecarKind::Nfo,
                });
            } else if ARTWORK_EXTENSIONS.contains(&ext.as_str()) {
                sidecars.push(DiscoveredSidecar {
                    old_path: path.clone(),
                    new_filename: file_name.to_string(),
                    kind: SidecarKind::Artwork,
                });
            }
        }
    }

    sidecars
}
