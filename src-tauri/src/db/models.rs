use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRoot {
    pub id: i64,
    pub path: String,
    pub media_type: String,
    pub enabled: i32,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: i64,
    pub library_root_id: i64,
    pub folder_path: String,
    pub file_path: String,
    pub title: String,
    pub sort_title: Option<String>,
    pub year: Option<i32>,
    pub resolution: Option<String>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub has_nfo: i32,
    pub has_poster: i32,
    pub has_backdrop: i32,
    pub file_size: Option<i64>,
    pub subtitle_count: i32,
    pub version_label: Option<String>,
    pub scan_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvShow {
    pub id: i64,
    pub library_root_id: i64,
    pub folder_path: String,
    pub title: String,
    pub sort_title: Option<String>,
    pub year: Option<i32>,
    pub tvdb_id: Option<i64>,
    pub tmdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub status: Option<String>,
    pub has_nfo: i32,
    pub has_poster: i32,
    pub has_backdrop: i32,
    pub season_count: i32,
    pub episode_count: i32,
    pub scan_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub tv_show_id: i64,
    pub file_path: String,
    pub title: Option<String>,
    pub season: i32,
    pub episode: i32,
    pub episode_end: Option<i32>,
    pub resolution: Option<String>,
    pub imdb_id: Option<String>,
    pub tvdb_id: Option<i64>,
    pub has_nfo: i32,
    pub file_size: Option<i64>,
    pub subtitle_count: i32,
    pub scan_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtitle {
    pub id: i64,
    pub media_type: String,
    pub media_id: i64,
    pub file_path: String,
    pub language: Option<String>,
    pub flags: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameLog {
    pub id: i64,
    pub batch_id: String,
    pub old_path: String,
    pub new_path: String,
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStreamInfo {
    pub id: i64,
    pub media_type: String,
    pub media_id: i64,
    pub file_path: String,
    pub container: Option<String>,
    pub video_codec: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub aspect_ratio: Option<String>,
    pub hdr_format: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<i32>,
    pub audio_layout: Option<String>,
    pub audio_languages: Option<String>,
    pub subtitle_languages: Option<String>,
    pub bitrate_kbps: Option<i32>,
    pub duration_sec: Option<f64>,
    pub probe_status: String,
    pub probed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateMovieItem {
    pub id: i64,
    pub library_root_id: i64,
    pub folder_path: String,
    pub file_path: String,
    pub title: String,
    pub year: Option<i32>,
    pub resolution: Option<String>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub file_size: Option<i64>,
    pub version_label: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub bitrate_kbps: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDuplicateGroup {
    pub match_key: String,
    pub reason: String,
    pub items: Vec<DuplicateMovieItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateEpisodeItem {
    pub id: i64,
    pub tv_show_id: i64,
    pub show_title: String,
    pub file_path: String,
    pub season: i32,
    pub episode: i32,
    pub title: Option<String>,
    pub resolution: Option<String>,
    pub file_size: Option<i64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub bitrate_kbps: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeDuplicateGroup {
    pub tv_show_id: i64,
    pub show_title: String,
    pub season: i32,
    pub episode: i32,
    pub match_key: String,
    pub items: Vec<DuplicateEpisodeItem>,
}
