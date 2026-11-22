use tauri::State;
use crate::DbState;
use crate::db::models::{Episode, Movie, TvShow};
use crate::db::queries;

#[tauri::command]
pub fn get_movies(
    library_root_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<Movie>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_movies(&conn, library_root_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_movie_detail(
    id: i64,
    state: State<'_, DbState>,
) -> Result<Option<Movie>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_movie_by_id(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tv_shows(
    library_root_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<TvShow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_tv_shows(&conn, library_root_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tv_show_detail(
    id: i64,
    state: State<'_, DbState>,
) -> Result<Option<TvShow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_tv_show_by_id(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_episodes(
    tv_show_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<Episode>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    queries::get_episodes_for_show(&conn, tv_show_id).map_err(|e| e.to_string())
}
