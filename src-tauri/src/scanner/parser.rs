use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Movie,
    Episode,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMedia {
    pub raw_filename: String,
    pub title: Option<String>,
    pub year: Option<u16>,
    pub resolution: Option<String>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub episode_end: Option<u16>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u32>,
    pub tvdb_id: Option<u32>,
    pub media_type: MediaType,
    pub version_label: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SidecarInfo {
    pub has_nfo: bool,
    pub has_poster: bool,
    pub has_backdrop: bool,
    pub subtitle_count: usize,
    pub subtitle_files: Vec<String>,
}

static PROVIDER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[(imdbid|tmdbid|tvdbid)-([^\]]+)\]").unwrap()
});

static SXXEXX_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)[Ss](\d{2,3})[Ee](\d{2,3})(?:-[Ee](\d{2,3}))?").unwrap()
});

static YEAR_PAREN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\((\d{4})\)").unwrap()
});

static YEAR_STANDALONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b((?:19|20)\d{2})\b").unwrap()
});

static RES_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(720p|1080p|2160p|4k)\b").unwrap()
});

static SCENE_GARBAGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(bluray|brrip|web-dl|webrip|hdtv|dvdrip|uhd|remux|hdr|x264|x265|hevc|h264|h265|aac|ac3|dts|ddp5\.1|atmos)\b").unwrap()
});

/// Parses a directory or file name into structured Jellyfin-compatible metadata.
pub fn parse_filename(filename: &str) -> ParsedMedia {
    // Strip file extension if present
    let stem = match filename.rfind('.') {
        Some(idx) if idx > 0 && idx < filename.len() - 1 => {
            let ext = &filename[idx + 1..].to_lowercase();
            if ["mkv", "mp4", "avi", "m4v", "ts", "wmv", "mov", "flv", "webm"].contains(&ext.as_str()) {
                &filename[..idx]
            } else {
                filename
            }
        }
        _ => filename,
    };

    let mut imdb_id: Option<String> = None;
    let mut tmdb_id: Option<u32> = None;
    let mut tvdb_id: Option<u32> = None;

    // 1. Extract Provider IDs
    for cap in PROVIDER_REGEX.captures_iter(stem) {
        let p_type = &cap[1];
        let p_val = &cap[2];
        match p_type {
            "imdbid" => imdb_id = Some(p_val.to_string()),
            "tmdbid" => tmdb_id = p_val.parse::<u32>().ok(),
            "tvdbid" => tvdb_id = p_val.parse::<u32>().ok(),
            _ => {}
        }
    }

    // 2. Extract Season & Episode
    let mut season: Option<u16> = None;
    let mut episode: Option<u16> = None;
    let mut episode_end: Option<u16> = None;

    if let Some(cap) = SXXEXX_REGEX.captures(stem) {
        season = cap.get(1).and_then(|m| m.as_str().parse().ok());
        episode = cap.get(2).and_then(|m| m.as_str().parse().ok());
        episode_end = cap.get(3).and_then(|m| m.as_str().parse().ok());
    }

    // 3. Extract Year
    let mut year: Option<u16> = None;
    let mut year_start_idx: Option<usize> = None;

    // Prioritize parenthesized (YYYY) first so titles containing years
    // (e.g., "1917 (2019)" or "Blade Runner 2049 (2017)") aren't truncated.
    let mut last_paren_cap = None;
    for cap in YEAR_PAREN_REGEX.captures_iter(stem) {
        last_paren_cap = Some(cap);
    }

    if let Some(cap) = last_paren_cap {
        if let Some(m) = cap.get(1) {
            year = m.as_str().parse().ok();
            year_start_idx = cap.get(0).map(|m0| m0.start());
        }
    } else {
        // If no parenthesized year, take the last standalone year match
        let mut last_standalone_cap = None;
        for cap in YEAR_STANDALONE_REGEX.captures_iter(stem) {
            last_standalone_cap = Some(cap);
        }
        if let Some(cap) = last_standalone_cap {
            if let Some(m) = cap.get(1) {
                year = m.as_str().parse().ok();
                year_start_idx = cap.get(0).map(|m0| m0.start());
            }
        }
    }

    // 4. Extract Resolution
    let mut resolution: Option<String> = None;
    if let Some(cap) = RES_REGEX.captures(stem) {
        if let Some(m) = cap.get(1) {
            let res_str = m.as_str().to_lowercase();
            resolution = Some(match res_str.as_str() {
                "4k" => "2160p".to_string(),
                _ => res_str,
            });
        }
    }

    // 5. Extract Multi-Version Label: e.g. "Blade Runner (1982) [id] - 1080p Theatrical"
    let mut version_label: Option<String> = None;
    if let Some(dash_idx) = stem.rfind(" - ") {
        let after_dash = &stem[dash_idx + 3..];
        // If after dash has provider ID or year, it's not a version label
        if !after_dash.contains("[imdbid-") && !after_dash.contains("[tmdbid-") {
            let clean_label = after_dash.trim();
            if !clean_label.is_empty() && (clean_label.contains("Cut") || clean_label.contains("Edition") || clean_label.contains("Theatrical") || clean_label.contains("Director") || RES_REGEX.is_match(clean_label)) {
                version_label = Some(clean_label.to_string());
            }
        }
    }

    // 6. Extract Title
    let mut cut_idx = stem.len();

    // Provider ID tag position
    if let Some(m) = PROVIDER_REGEX.find(stem) {
        if m.start() < cut_idx {
            cut_idx = m.start();
        }
    }

    // Year position
    if let Some(idx) = year_start_idx {
        if idx < cut_idx {
            cut_idx = idx;
        }
    }

    // SXXEXX position
    if let Some(m) = SXXEXX_REGEX.find(stem) {
        if m.start() < cut_idx {
            cut_idx = m.start();
        }
    }

    // Scene tags position
    if let Some(m) = SCENE_GARBAGE_REGEX.find(stem) {
        if m.start() < cut_idx {
            cut_idx = m.start();
        }
    }

    // Resolution position
    if let Some(m) = RES_REGEX.find(stem) {
        if m.start() < cut_idx {
            cut_idx = m.start();
        }
    }

    let raw_title = &stem[..cut_idx];
    let mut cleaned_title = raw_title.replace('.', " ").replace('_', " ");
    
    // Clean trailing hyphens or brackets
    cleaned_title = cleaned_title.trim().trim_end_matches('-').trim().to_string();

    let title = if !cleaned_title.is_empty() {
        Some(cleaned_title)
    } else {
        None
    };

    // Determine MediaType
    let media_type = if season.is_some() && episode.is_some() {
        MediaType::Episode
    } else if year.is_some() || imdb_id.is_some() || tmdb_id.is_some() {
        MediaType::Movie
    } else {
        MediaType::Unknown
    };

    ParsedMedia {
        raw_filename: filename.to_string(),
        title,
        year,
        resolution,
        season,
        episode,
        episode_end,
        imdb_id,
        tmdb_id,
        tvdb_id,
        media_type,
        version_label,
    }
}

/// Detects sidecar files in a directory for a given video file stem.
pub fn detect_sidecars(dir: &Path, video_stem: &str) -> SidecarInfo {
    let mut info = SidecarInfo::default();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return info,
    };

    let sub_exts = ["srt", "sub", "ass", "ssa", "vtt", "sup"];
    let stem_lower = video_stem.to_lowercase();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        let file_lower = file_name.to_lowercase();

        // Check NFO
        if file_lower == "movie.nfo"
            || file_lower == "tvshow.nfo"
            || file_lower == format!("{}.nfo", stem_lower)
        {
            info.has_nfo = true;
        }

        // Check Poster
        if file_lower.starts_with("poster.") || file_lower.starts_with("folder.") || file_lower.starts_with("cover.") {
            info.has_poster = true;
        }

        // Check Backdrop / Fanart
        if file_lower.starts_with("backdrop.") || file_lower.starts_with("fanart.") {
            info.has_backdrop = true;
        }

        // Check Subtitles
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if sub_exts.contains(&ext.as_str()) && file_lower.starts_with(&stem_lower) {
            info.subtitle_count += 1;
            info.subtitle_files.push(file_name.to_string());
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_jellyfin_movie() {
        let parsed = parse_filename("Alien (1979) [imdbid-tt0078748]");
        assert_eq!(parsed.title.as_deref(), Some("Alien"));
        assert_eq!(parsed.year, Some(1979));
        assert_eq!(parsed.imdb_id.as_deref(), Some("tt0078748"));
        assert_eq!(parsed.media_type, MediaType::Movie);
    }

    #[test]
    fn test_tmdb_movie_with_colon_replacement() {
        let parsed = parse_filename("Dune - Part Two (2024) [tmdbid-693134]");
        assert_eq!(parsed.title.as_deref(), Some("Dune - Part Two"));
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.tmdb_id, Some(693134));
        assert_eq!(parsed.media_type, MediaType::Movie);
    }

    #[test]
    fn test_accented_movie() {
        let parsed = parse_filename("Amélie (2001) [imdbid-tt0211915]");
        assert_eq!(parsed.title.as_deref(), Some("Amélie"));
        assert_eq!(parsed.year, Some(2001));
        assert_eq!(parsed.imdb_id.as_deref(), Some("tt0211915"));
    }

    #[test]
    fn test_multi_version_movie() {
        let parsed = parse_filename("Blade Runner (1982) [imdbid-tt0083658] - 2160p Final Cut.mkv");
        assert_eq!(parsed.title.as_deref(), Some("Blade Runner"));
        assert_eq!(parsed.year, Some(1982));
        assert_eq!(parsed.resolution.as_deref(), Some("2160p"));
        assert_eq!(parsed.version_label.as_deref(), Some("2160p Final Cut"));
    }

    #[test]
    fn test_messy_scene_movie() {
        let parsed = parse_filename("the.matrix.1999.2160p.BluRay.x265-SPARKS.mkv");
        assert_eq!(parsed.title.as_deref(), Some("the matrix"));
        assert_eq!(parsed.year, Some(1999));
        assert_eq!(parsed.resolution.as_deref(), Some("2160p"));
        assert_eq!(parsed.media_type, MediaType::Movie);
    }

    #[test]
    fn test_tv_episode_clean() {
        let parsed = parse_filename("Breaking Bad S01E01 - Pilot.mkv");
        assert_eq!(parsed.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(parsed.season, Some(1));
        assert_eq!(parsed.episode, Some(1));
        assert_eq!(parsed.episode_end, None);
        assert_eq!(parsed.media_type, MediaType::Episode);
    }

    #[test]
    fn test_tv_multi_episode() {
        let parsed = parse_filename("Breaking Bad S05E15-E16 - Granite State & Felina.mkv");
        assert_eq!(parsed.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(parsed.season, Some(5));
        assert_eq!(parsed.episode, Some(15));
        assert_eq!(parsed.episode_end, Some(16));
        assert_eq!(parsed.media_type, MediaType::Episode);
    }

    #[test]
    fn test_movie_with_year_in_title_1917() {
        let parsed = parse_filename("1917 (2019) [imdbid-tt8579674]");
        assert_eq!(parsed.title.as_deref(), Some("1917"));
        assert_eq!(parsed.year, Some(2019));
        assert_eq!(parsed.imdb_id.as_deref(), Some("tt8579674"));
        assert_eq!(parsed.media_type, MediaType::Movie);
    }

    #[test]
    fn test_movie_with_year_in_title_blade_runner_2049() {
        let parsed = parse_filename("Blade Runner 2049 (2017) [imdbid-tt1856101]");
        assert_eq!(parsed.title.as_deref(), Some("Blade Runner 2049"));
        assert_eq!(parsed.year, Some(2017));
        assert_eq!(parsed.imdb_id.as_deref(), Some("tt1856101"));
        assert_eq!(parsed.media_type, MediaType::Movie);
    }

    #[test]
    fn test_movie_scene_with_year_in_title() {
        let parsed = parse_filename("Blade.Runner.2049.2017.1080p.BluRay.x264.mkv");
        assert_eq!(parsed.title.as_deref(), Some("Blade Runner 2049"));
        assert_eq!(parsed.year, Some(2017));
        assert_eq!(parsed.resolution.as_deref(), Some("1080p"));
    }
}
