use tauri::State;
use crate::DbState;
use crate::db::queries::get_library_root_by_id;
use crate::scanner::walker::{scan_movie_root, scan_tvshow_root};
use crate::scanner::ScanResult;

#[tauri::command]
pub async fn scan_library(
    root_id: i64,
    state: State<'_, DbState>,
    app_handle: tauri::AppHandle,
) -> Result<ScanResult, String> {
    let (path, media_type) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let root = get_library_root_by_id(&conn, root_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Library root {} not found", root_id))?;
        (root.path, root.media_type)
    };

    let result = if media_type == "tvshow" {
        scan_tvshow_root(&path, root_id, &state.0, Some(&app_handle))
    } else {
        scan_movie_root(&path, root_id, &state.0, Some(&app_handle))
    };

    Ok(result)
}
