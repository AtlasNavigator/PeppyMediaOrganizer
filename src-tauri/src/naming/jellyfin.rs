use crate::naming::sanitize::sanitize_path_component;

/// Formats a movie folder name according to Jellyfin standards:
/// `Movie Name (Year) [imdbid-tt1234567]` or `Movie Name (Year) [tmdbid-12345]`
pub fn format_movie_folder(title: &str, year: Option<u16>, provider_id_tag: Option<&str>) -> String {
    let clean_title = sanitize_path_component(title);
    let mut parts = vec![clean_title];

    if let Some(y) = year {
        parts.push(format!("({})", y));
    }

    if let Some(tag) = provider_id_tag {
        if !tag.is_empty() {
            parts.push(tag.to_string());
        }
    }

    parts.join(" ")
}

/// Formats a movie video filename according to Jellyfin standards.
/// Prefix matches the parent folder name exactly.
/// If multiple versions exist in the same folder, appends ` - Label` (e.g. ` - 1080p`).
pub fn format_movie_filename(
    folder_name: &str,
    version_label: Option<&str>,
    extension: &str,
) -> String {
    let ext = extension.trim_start_matches('.');
    if let Some(label) = version_label {
        let clean_label = sanitize_path_component(label);
        if !clean_label.is_empty() {
            return format!("{} - {}.{}", folder_name, clean_label, ext);
        }
    }
    format!("{}.{}", folder_name, ext)
}

/// Constructs the primary Jellyfin provider ID bracket tag:
/// `[imdbid-tt1234567]`, `[tmdbid-12345]`, or `[tvdbid-12345]`.
pub fn format_provider_tag(
    imdb_id: Option<&str>,
    tmdb_id: Option<u32>,
    tvdb_id: Option<u32>,
    is_movie: bool,
) -> Option<String> {
    if is_movie {
        if let Some(imdb) = imdb_id {
            let clean = if imdb.starts_with("tt") {
                imdb.to_string()
            } else {
                format!("tt{}", imdb)
            };
            return Some(format!("[imdbid-{}]", clean));
        }
        if let Some(tmdb) = tmdb_id {
            return Some(format!("[tmdbid-{}]", tmdb));
        }
        if let Some(tvdb) = tvdb_id {
            return Some(format!("[tvdbid-{}]", tvdb));
        }
    } else {
        if let Some(tvdb) = tvdb_id {
            return Some(format!("[tvdbid-{}]", tvdb));
        }
        if let Some(tmdb) = tmdb_id {
            return Some(format!("[tmdbid-{}]", tmdb));
        }
        if let Some(imdb) = imdb_id {
            let clean = if imdb.starts_with("tt") {
                imdb.to_string()
            } else {
                format!("tt{}", imdb)
            };
            return Some(format!("[imdbid-{}]", clean));
        }
    }
    None
}

/// Formats a TV show series folder name:
/// `Series Name (Year) [tvdbid-12345]`
pub fn format_tv_show_folder(title: &str, year: Option<u16>, provider_id_tag: Option<&str>) -> String {
    let clean_title = sanitize_path_component(title);
    let mut parts = vec![clean_title];

    if let Some(y) = year {
        parts.push(format!("({})", y));
    }

    if let Some(tag) = provider_id_tag {
        if !tag.is_empty() {
            parts.push(tag.to_string());
        }
    }

    parts.join(" ")
}

/// Formats a TV show season folder:
/// `Season 01`, `Season 02`, `Season 00` for specials.
pub fn format_season_folder(season: u16) -> String {
    format!("Season {:02}", season)
}

/// Formats an episode filename:
/// `Series Name S01E01 - Episode Title.ext`
/// Zero-pads to 2 digits by default, or 3 digits if episode number > 99.
pub fn format_episode_filename(
    series_name: &str,
    season: u16,
    episode: u16,
    episode_end: Option<u16>,
    episode_title: Option<&str>,
    extension: &str,
) -> String {
    let clean_series = sanitize_path_component(series_name);
    let pad_digits = if episode > 99 || episode_end.unwrap_or(0) > 99 {
        3
    } else {
        2
    };

    let ep_tag = match episode_end {
        Some(end) => format!(
            "S{:02}E{:0width$}-E{:0width$}",
            season,
            episode,
            end,
            width = pad_digits
        ),
        None => format!("S{:02}E{:0width$}", season, episode, width = pad_digits),
    };

    let ext = extension.trim_start_matches('.');

    if let Some(title) = episode_title {
        let clean_title = sanitize_path_component(title);
        if !clean_title.is_empty() {
            return format!("{} {} - {}.{}", clean_series, ep_tag, clean_title, ext);
        }
    }

    format!("{} {}.{}", clean_series, ep_tag, ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_movie_folder() {
        assert_eq!(
            format_movie_folder("Alien", Some(1979), Some("[imdbid-tt0078748]")),
            "Alien (1979) [imdbid-tt0078748]"
        );
        assert_eq!(
            format_movie_folder("Dune: Part Two", Some(2024), Some("[tmdbid-693134]")),
            "Dune - Part Two (2024) [tmdbid-693134]"
        );
    }

    #[test]
    fn test_format_season_folder() {
        assert_eq!(format_season_folder(1), "Season 01");
        assert_eq!(format_season_folder(0), "Season 00");
    }

    #[test]
    fn test_format_episode_filename() {
        assert_eq!(
            format_episode_filename("Breaking Bad", 1, 1, None, Some("Pilot"), "mkv"),
            "Breaking Bad S01E01 - Pilot.mkv"
        );
        assert_eq!(
            format_episode_filename("Breaking Bad", 5, 15, Some(16), Some("Granite State & Felina"), "mkv"),
            "Breaking Bad S05E15-E16 - Granite State & Felina.mkv"
        );
    }
}
