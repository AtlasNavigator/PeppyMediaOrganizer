use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::renamer::plan::{ActionStatus, RenameItemType, RenamePlan};

pub static IS_RENAMING: AtomicBool = AtomicBool::new(false);

pub struct RenamingGuard;
impl RenamingGuard {
    pub fn new() -> Self {
        IS_RENAMING.store(true, Ordering::SeqCst);
        RenamingGuard
    }
}
impl Drop for RenamingGuard {
    fn drop(&mut self) {
        IS_RENAMING.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameExecutionResult {
    pub batch_id: String,
    pub executed_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResult {
    pub batch_id: String,
    pub reverted_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

/// Recursively copies a directory to another path.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let target_path = dst.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &target_path)?;
        } else {
            fs::copy(&entry_path, &target_path)?;
        }
    }
    Ok(())
}

/// Moves a file or directory safely, with graceful cross-filesystem (`EXDEV`) fallback:
/// copy + size verification + delete source.
pub fn safe_move_path(src: &Path, dst: &Path) -> io::Result<()> {
    if src == dst {
        return Ok(());
    }

    if !src.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source path does not exist: {}", src.display()),
        ));
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(err) => {
            // Check for EXDEV (error code 18 on Unix / CrossesDevices)
            let is_cross_device = err.kind() == io::ErrorKind::CrossesDevices
                || err.raw_os_error() == Some(18);

            if is_cross_device {
                if src.is_dir() {
                    copy_dir_recursive(src, dst)?;
                    fs::remove_dir_all(src)?;
                    Ok(())
                } else {
                    fs::copy(src, dst)?;
                    let src_len = fs::metadata(src)?.len();
                    let dst_len = fs::metadata(dst)?.len();
                    if src_len != dst_len {
                        let _ = fs::remove_file(dst);
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "File size mismatch during cross-filesystem move",
                        ));
                    }
                    fs::remove_file(src)?;
                    Ok(())
                }
            } else {
                Err(err)
            }
        }
    }
}

/// Executes confirmed actions from a `RenamePlan`.
pub fn execute_rename_plan(
    conn: &mut Connection,
    plan: &RenamePlan,
    selected_action_ids: Option<&[String]>,
) -> Result<RenameExecutionResult, String> {
    let _guard = RenamingGuard::new();
    let mut executed_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    // Determine actionable items
    let mut actions_to_run = Vec::new();
    for action in &plan.actions {
        if action.status == ActionStatus::Conflict || action.status == ActionStatus::Skipped {
            continue;
        }

        if let Some(ids) = selected_action_ids {
            if !ids.iter().any(|id| id == &action.id) {
                continue;
            }
        }

        if action.old_path != action.new_path {
            actions_to_run.push(action.clone());
        }
    }

    // Sort execution order: files first, then directories
    actions_to_run.sort_by(|a, b| {
        let is_dir_a = matches!(
            a.item_type,
            RenameItemType::MovieFolder | RenameItemType::TvShowFolder | RenameItemType::SeasonFolder
        );
        let is_dir_b = matches!(
            b.item_type,
            RenameItemType::MovieFolder | RenameItemType::TvShowFolder | RenameItemType::SeasonFolder
        );

        match (is_dir_a, is_dir_b) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });

    let mut old_dirs_to_clean: Vec<PathBuf> = Vec::new();

    for action in &actions_to_run {
        let src = Path::new(&action.old_path);
        let dst = Path::new(&action.new_path);

        if !src.exists() {
            failed_count += 1;
            errors.push(format!("Source does not exist: {}", action.old_path));
            continue;
        }

        // If directory action, record old path for cleanup after moving files
        let is_dir = matches!(
            action.item_type,
            RenameItemType::MovieFolder | RenameItemType::TvShowFolder | RenameItemType::SeasonFolder
        );

        if is_dir {
            // Just create the destination directory
            if let Err(e) = fs::create_dir_all(dst) {
                failed_count += 1;
                errors.push(format!("Failed to create directory {}: {}", dst.display(), e));
                continue;
            }
            old_dirs_to_clean.push(src.to_path_buf());
        } else {
            // Move file
            if let Err(e) = safe_move_path(src, dst) {
                failed_count += 1;
                errors.push(format!("Failed to move {}: {}", src.display(), e));
                continue;
            }
        }

        // Log to rename_log
        let type_str = format!("{:?}", action.item_type).to_lowercase();
        let _ = conn.execute(
            r#"
            INSERT INTO rename_log (
                batch_id, operation_type, media_type, media_id, old_path, new_path, item_type
            ) VALUES (?1, 'rename', ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                plan.batch_id,
                action.media_type,
                action.media_id,
                action.old_path,
                action.new_path,
                type_str,
            ],
        );

        // Update database records
        match action.item_type {
            RenameItemType::MovieFolder => {
                let _ = conn.execute(
                    "UPDATE movies SET folder_path = ?1, updated_at = datetime('now') WHERE id = ?2",
                    params![action.new_path, action.media_id],
                );
            }
            RenameItemType::MovieVideo => {
                let _ = conn.execute(
                    "UPDATE movies SET file_path = ?1, updated_at = datetime('now') WHERE id = ?2",
                    params![action.new_path, action.media_id],
                );
            }
            RenameItemType::TvShowFolder => {
                let _ = conn.execute(
                    "UPDATE tv_shows SET folder_path = ?1, updated_at = datetime('now') WHERE id = ?2",
                    params![action.new_path, action.media_id],
                );
            }
            RenameItemType::EpisodeVideo => {
                let _ = conn.execute(
                    "UPDATE episodes SET file_path = ?1, updated_at = datetime('now') WHERE file_path = ?2",
                    params![action.new_path, action.old_path],
                );
            }
            RenameItemType::Subtitle => {
                let _ = conn.execute(
                    "UPDATE subtitles SET file_path = ?1 WHERE file_path = ?2",
                    params![action.new_path, action.old_path],
                );
            }
            _ => {}
        }

        executed_count += 1;
    }

    // Clean up empty old directories (deepest first)
    old_dirs_to_clean.sort_by_key(|b| std::cmp::Reverse(b.components().count()));
    for dir in old_dirs_to_clean {
        if dir.exists() {
            let is_empty = fs::read_dir(&dir).map(|mut d| d.next().is_none()).unwrap_or(false);
            if is_empty {
                let _ = fs::remove_dir(&dir);
            }
        }
    }

    Ok(RenameExecutionResult {
        batch_id: plan.batch_id.clone(),
        executed_count,
        failed_count,
        errors,
    })
}

/// Reverts the most recent batch rename operation recorded in `rename_log`.
pub fn undo_last_rename(conn: &mut Connection) -> Result<UndoResult, String> {
    let _guard = RenamingGuard::new();
    // 1. Find latest unreverted batch_id
    let batch_id: Option<String> = conn
        .query_row(
            r#"
            SELECT batch_id FROM rename_log
            WHERE reverted_at IS NULL AND operation_type = 'rename'
            ORDER BY id DESC LIMIT 1
            "#,
            [],
            |r| r.get(0),
        )
        .ok();

    let batch_id = match batch_id {
        Some(b) => b,
        None => return Err("No rename batch found to undo".into()),
    };

    // 2. Fetch all entries for this batch in reverse order (ID descending)
    #[allow(dead_code)]
    struct HistRow {
        id: i64,
        media_type: String,
        media_id: Option<i64>,
        old_path: String,
        new_path: String,
        item_type: String,
    }

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, media_type, media_id, old_path, new_path, item_type
            FROM rename_log
            WHERE batch_id = ?1 AND reverted_at IS NULL
            ORDER BY id DESC
            "#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![batch_id], |row| {
            Ok(HistRow {
                id: row.get(0)?,
                media_type: row.get(1)?,
                media_id: row.get(2)?,
                old_path: row.get(3)?,
                new_path: row.get(4)?,
                item_type: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for r in rows {
        entries.push(r.map_err(|e| e.to_string())?);
    }

    let mut reverted_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();
    let mut dirs_to_clean: Vec<PathBuf> = Vec::new();

    for entry in &entries {
        let current_path = Path::new(&entry.new_path);
        let original_path = Path::new(&entry.old_path);

        if !current_path.exists() {
            failed_count += 1;
            errors.push(format!("File to revert does not exist: {}", entry.new_path));
            continue;
        }

        let is_dir = entry.item_type.contains("folder");

        if is_dir {
            // Re-create old directory
            if let Err(e) = fs::create_dir_all(original_path) {
                failed_count += 1;
                errors.push(format!("Failed to recreate directory {}: {}", original_path.display(), e));
                continue;
            }
            dirs_to_clean.push(current_path.to_path_buf());
        } else {
            // Revert file move
            if let Err(e) = safe_move_path(current_path, original_path) {
                failed_count += 1;
                errors.push(format!("Failed to revert file {}: {}", current_path.display(), e));
                continue;
            }
        }

        // Revert database records
        if entry.item_type == "moviefolder" {
            let _ = conn.execute(
                "UPDATE movies SET folder_path = ?1, updated_at = datetime('now') WHERE folder_path = ?2",
                params![entry.old_path, entry.new_path],
            );
        } else if entry.item_type == "movievideo" {
            let _ = conn.execute(
                "UPDATE movies SET file_path = ?1, updated_at = datetime('now') WHERE file_path = ?2",
                params![entry.old_path, entry.new_path],
            );
        } else if entry.item_type == "tvshowfolder" {
            let _ = conn.execute(
                "UPDATE tv_shows SET folder_path = ?1, updated_at = datetime('now') WHERE folder_path = ?2",
                params![entry.old_path, entry.new_path],
            );
        } else if entry.item_type == "episodevideo" {
            let _ = conn.execute(
                "UPDATE episodes SET file_path = ?1, updated_at = datetime('now') WHERE file_path = ?2",
                params![entry.old_path, entry.new_path],
            );
        } else if entry.item_type == "subtitle" {
            let _ = conn.execute(
                "UPDATE subtitles SET file_path = ?1 WHERE file_path = ?2",
                params![entry.old_path, entry.new_path],
            );
        }

        reverted_count += 1;
    }

    // Clean up empty directories from the reverted batch
    dirs_to_clean.sort_by_key(|b| std::cmp::Reverse(b.components().count()));
    for dir in dirs_to_clean {
        if dir.exists() {
            let is_empty = fs::read_dir(&dir).map(|mut d| d.next().is_none()).unwrap_or(false);
            if is_empty {
                let _ = fs::remove_dir(&dir);
            }
        }
    }

    // Mark batch as reverted
    let _ = conn.execute(
        "UPDATE rename_log SET reverted_at = datetime('now') WHERE batch_id = ?1",
        params![batch_id],
    );

    Ok(UndoResult {
        batch_id,
        reverted_count,
        failed_count,
        errors,
    })
}
