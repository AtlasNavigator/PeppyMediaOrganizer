use std::path::Path;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use tauri::Emitter;

use crate::db::models::MediaStreamInfo;
use crate::db::queries::{get_pending_probe_paths, upsert_media_stream};
use crate::media_info::probe::{detect_ffprobe_binary, run_ffprobe};

/// Probes a single media file directly and stores the result in SQLite.
pub fn probe_single_file(
    db: &Arc<Mutex<Connection>>,
    file_path: &str,
    media_type: &str,
    media_id: i64,
    configured_ffprobe: Option<&str>,
) -> Result<MediaStreamInfo, String> {
    let binary = detect_ffprobe_binary(configured_ffprobe)
        .ok_or_else(|| "ffprobe binary not found. Please install ffmpeg or configure path in Settings.".to_string())?;

    let path = Path::new(file_path);
    let stream_info = match run_ffprobe(&binary, path, media_type, media_id) {
        Ok(info) => info,
        Err(e) => {
            log::warn!("ffprobe failed for {}: {}", file_path, e);
            // Record failed probe status so we don't spin indefinitely
            let failed_info = MediaStreamInfo {
                id: 0,
                media_type: media_type.to_string(),
                media_id,
                file_path: file_path.to_string(),
                container: None,
                video_codec: None,
                width: None,
                height: None,
                aspect_ratio: None,
                hdr_format: None,
                audio_codec: None,
                audio_channels: None,
                audio_layout: None,
                audio_languages: None,
                subtitle_languages: None,
                bitrate_kbps: None,
                duration_sec: None,
                probe_status: "failed".to_string(),
                probed_at: None,
            };
            if let Ok(conn) = db.lock() {
                let _ = upsert_media_stream(&conn, &failed_info);
            }
            return Err(e);
        }
    };

    if let Ok(conn) = db.lock() {
        let _ = upsert_media_stream(&conn, &stream_info);
    }

    Ok(stream_info)
}

/// Processes a batch of pending un-probed media files in the background.
pub fn probe_pending_batch(
    db: &Arc<Mutex<Connection>>,
    app_handle: Option<&tauri::AppHandle>,
    configured_ffprobe: Option<&str>,
    batch_size: usize,
) -> usize {
    let binary = match detect_ffprobe_binary(configured_ffprobe) {
        Some(b) => b,
        None => return 0,
    };

    let pending: Vec<(String, String, i64)> = {
        let conn = match db.lock() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        get_pending_probe_paths(&conn, batch_size).unwrap_or_default()
    };

    let mut probed_count = 0;

    for (file_path, media_type, media_id) in pending {
        let path = Path::new(&file_path);
        let stream_info = match run_ffprobe(&binary, path, &media_type, media_id) {
            Ok(info) => info,
            Err(e) => {
                log::debug!("Failed to probe {}: {}", file_path, e);
                MediaStreamInfo {
                    id: 0,
                    media_type: media_type.clone(),
                    media_id,
                    file_path: file_path.clone(),
                    container: None,
                    video_codec: None,
                    width: None,
                    height: None,
                    aspect_ratio: None,
                    hdr_format: None,
                    audio_codec: None,
                    audio_channels: None,
                    audio_layout: None,
                    audio_languages: None,
                    subtitle_languages: None,
                    bitrate_kbps: None,
                    duration_sec: None,
                    probe_status: "failed".to_string(),
                    probed_at: None,
                }
            }
        };

        if let Ok(conn) = db.lock() {
            if upsert_media_stream(&conn, &stream_info).is_ok() {
                probed_count += 1;
            }
        }

        if let Some(handle) = app_handle {
            let _ = handle.emit("media:stream_probed", &stream_info);
        }
    }

    probed_count
}
