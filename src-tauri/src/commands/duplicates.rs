use tauri::State;

use crate::db::models::{EpisodeDuplicateGroup, MovieDuplicateGroup};
use crate::duplicates;
use crate::DbState;

#[tauri::command]
pub fn get_movie_duplicates(state: State<'_, DbState>) -> Result<Vec<MovieDuplicateGroup>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    duplicates::get_movie_duplicates(&conn)
}

#[tauri::command]
pub fn get_episode_duplicates(
    state: State<'_, DbState>,
) -> Result<Vec<EpisodeDuplicateGroup>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    duplicates::get_episode_duplicates(&conn)
}
