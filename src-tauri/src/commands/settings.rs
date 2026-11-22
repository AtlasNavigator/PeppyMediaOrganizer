use tauri::State;
use crate::DbState;
use crate::db::models::LibraryRoot;
use crate::db::queries;

#[tauri::command]
pub fn add_library_root(
    path: String,
    media_type: String,
    state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::insert_library_root(&conn, &path, &media_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_library_roots(
    state: State<'_, DbState>,
) -> Result<Vec<LibraryRoot>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_library_roots(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_library_root(
    id: i64,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::delete_library_root(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_library_root_enabled(
    id: i64,
    enabled: bool,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::toggle_library_root(&conn, id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_setting(
    key: String,
    state: State<'_, DbState>,
) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_setting(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_setting(
    key: String,
    value: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_system_text_scale() -> Result<f64, String> {
    if let Ok(val) = std::env::var("GDK_DPI_SCALE") {
        if let Ok(scale) = val.trim().parse::<f64>() {
            if scale > 0.1 && scale < 10.0 {
                return Ok(scale);
            }
        }
    }
    if let Ok(val) = std::env::var("QT_SCALE_FACTOR") {
        if let Ok(scale) = val.trim().parse::<f64>() {
            if scale > 0.1 && scale < 10.0 {
                return Ok(scale);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "text-scaling-factor"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Ok(scale) = text.trim().parse::<f64>() {
                    if scale > 0.1 && scale < 10.0 {
                        return Ok(scale);
                    }
                }
            }
        }
    }
    Ok(1.0)
}

#[tauri::command]
pub fn set_window_zoom(window: tauri::WebviewWindow, zoom: f64) -> Result<(), String> {
    window.set_zoom(zoom).map_err(|e| e.to_string())
}

