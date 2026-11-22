use tauri::State;

use crate::db::queries;
use crate::scanner::watcher::WatcherStatus;
use crate::{DbState, WatcherState};

#[tauri::command]
pub fn get_watcher_status(
    watcher_state: State<'_, WatcherState>,
) -> Result<WatcherStatus, String> {
    let watcher = watcher_state.0.lock().map_err(|e| e.to_string())?;
    Ok(watcher.get_status())
}

#[tauri::command]
pub fn set_watcher_enabled(
    enabled: bool,
    watcher_state: State<'_, WatcherState>,
    db_state: State<'_, DbState>,
    app_handle: tauri::AppHandle,
) -> Result<WatcherStatus, String> {
    let roots = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        queries::set_setting(&conn, "auto_watch_enabled", if enabled { "1" } else { "0" })
            .map_err(|e| e.to_string())?;
        queries::get_library_roots(&conn).map_err(|e| e.to_string())?
    };

    let mut watcher = watcher_state.0.lock().map_err(|e| e.to_string())?;
    watcher.set_enabled(enabled, roots, Some(app_handle), db_state.0.clone())?;
    Ok(watcher.get_status())
}

#[tauri::command]
pub fn reload_watched_roots(
    watcher_state: State<'_, WatcherState>,
    db_state: State<'_, DbState>,
    app_handle: tauri::AppHandle,
) -> Result<WatcherStatus, String> {
    let roots = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        queries::get_library_roots(&conn).map_err(|e| e.to_string())?
    };

    let mut watcher = watcher_state.0.lock().map_err(|e| e.to_string())?;
    watcher.reload(roots, Some(app_handle), db_state.0.clone())?;
    Ok(watcher.get_status())
}
