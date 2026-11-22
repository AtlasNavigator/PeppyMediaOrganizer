use tauri::State;

use crate::db::models::MediaStreamInfo;
use crate::db::queries;
use crate::media_info::probe::detect_ffprobe_binary;
use crate::media_info::worker::{probe_pending_batch as worker_probe_batch, probe_single_file};
use crate::DbState;

#[tauri::command]
pub fn get_media_stream_info(
    media_type: String,
    media_id: i64,
    state: State<'_, DbState>,
) -> Result<Option<MediaStreamInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_media_stream_by_media(&conn, &media_type, media_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn probe_media_file(
    file_path: String,
    media_type: String,
    media_id: i64,
    state: State<'_, DbState>,
) -> Result<MediaStreamInfo, String> {
    let configured_path = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        queries::get_setting(&conn, "ffprobe_path").ok().flatten()
    };

    probe_single_file(
        &state.0,
        &file_path,
        &media_type,
        media_id,
        configured_path.as_deref(),
    )
}

#[tauri::command]
pub fn probe_pending_batch(
    batch_size: Option<usize>,
    state: State<'_, DbState>,
    app_handle: tauri::AppHandle,
) -> Result<usize, String> {
    let configured_path = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        queries::get_setting(&conn, "ffprobe_path").ok().flatten()
    };

    let count = worker_probe_batch(
        &state.0,
        Some(&app_handle),
        configured_path.as_deref(),
        batch_size.unwrap_or(10),
    );

    Ok(count)
}

#[tauri::command]
pub fn get_ffprobe_path(state: State<'_, DbState>) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let configured = queries::get_setting(&conn, "ffprobe_path").ok().flatten();
    let detected = detect_ffprobe_binary(configured.as_deref());
    Ok(configured.or(detected))
}

#[tauri::command]
pub fn set_ffprobe_path(path: String, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::set_setting(&conn, "ffprobe_path", &path).map_err(|e| e.to_string())
}
