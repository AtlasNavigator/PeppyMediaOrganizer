use tauri::State;
use crate::DbState;
use crate::renamer::{
    execute_rename_plan as exec_plan, plan_batch_rename as p_batch, plan_movie_rename as p_movie,
    plan_resolve_duplicates as p_resolve_dup, plan_tv_show_rename as p_tv, undo_last_rename as undo_rename,
    RenameExecutionResult, RenamePlan, UndoResult,
};

#[tauri::command]
pub fn plan_rename_movie(movie_id: i64, state: State<'_, DbState>) -> Result<RenamePlan, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    p_movie(&conn, movie_id)
}

#[tauri::command]
pub fn plan_rename_tv_show(tv_show_id: i64, state: State<'_, DbState>) -> Result<RenamePlan, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    p_tv(&conn, tv_show_id)
}

#[tauri::command]
pub fn plan_batch_rename(
    movie_ids: Vec<i64>,
    tv_show_ids: Vec<i64>,
    state: State<'_, DbState>,
) -> Result<RenamePlan, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    p_batch(&conn, &movie_ids, &tv_show_ids)
}

#[tauri::command]
pub fn plan_resolve_duplicates(
    keep_file_path: String,
    obsolete_file_paths: Vec<String>,
    media_type: String,
    state: State<'_, DbState>,
) -> Result<RenamePlan, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    p_resolve_dup(&conn, &keep_file_path, &obsolete_file_paths, &media_type)
}

#[tauri::command]
pub fn execute_rename_plan(
    plan: RenamePlan,
    selected_action_ids: Option<Vec<String>>,
    state: State<'_, DbState>,
) -> Result<RenameExecutionResult, String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    exec_plan(&mut conn, &plan, selected_action_ids.as_deref())
}

#[tauri::command]
pub fn undo_last_rename(state: State<'_, DbState>) -> Result<UndoResult, String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    undo_rename(&mut conn)
}

#[tauri::command]
pub fn can_undo(state: State<'_, DbState>) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM rename_log WHERE reverted_at IS NULL AND operation_type = 'rename'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Ok(count > 0)
}
