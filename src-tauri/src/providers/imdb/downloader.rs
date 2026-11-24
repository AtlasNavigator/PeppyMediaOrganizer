use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
use tauri::Emitter;

use super::indexer::{self, IndexProgress};

static IS_DOWNLOADING: AtomicBool = AtomicBool::new(false);

/// RAII guard to ensure IS_DOWNLOADING is always reset to false on drop.
struct DownloadGuard;
impl Drop for DownloadGuard {
    fn drop(&mut self) {
        IS_DOWNLOADING.store(false, Ordering::SeqCst);
    }
}

/// Returns whether an IMDb download/indexing operation is currently active.
pub fn is_downloading() -> bool {
    IS_DOWNLOADING.load(Ordering::SeqCst)
}

/// Removes any leftover temporary files or folders in `app_data_dir/imdb_temp`.
pub fn clean_temp_dir(app_data_dir: &Path) {
    let temp_dir = app_data_dir.join("imdb_temp");
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

/// Checks whether `imdb.db` has indexed records.
pub fn is_imdb_installed(conn: &Connection) -> bool {
    let count: i64 = conn
        .query_row("SELECT count(*) FROM imdb_titles", [], |r| r.get(0))
        .unwrap_or(0);
    count > 0
}

/// Checks whether `imdb.db` is older than 7 days (or not yet synced).
pub fn is_imdb_stale(conn: &Connection) -> bool {
    let last_synced: Result<Option<String>, _> = conn.query_row(
        "SELECT last_synced_at FROM imdb_sync_status WHERE dataset_name = 'title.basics'",
        [],
        |r| r.get(0),
    );

    match last_synced {
        Ok(Some(synced_str)) => {
            // First try SQLite julianday diff if valid
            let days_old: Result<f64, _> = conn.query_row(
                "SELECT julianday('now') - julianday(?1)",
                params![synced_str],
                |r| r.get(0),
            );

            if let Ok(days) = days_old {
                days > 7.0
            } else {
                // Fallback to chrono parsing
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&synced_str, "%Y-%m-%d %H:%M:%S") {
                    let duration = chrono::Utc::now().naive_utc().signed_duration_since(dt);
                    duration.num_days() >= 7
                } else {
                    true
                }
            }
        }
        // No sync record or NULL timestamp means it needs refresh
        _ => true,
    }
}

/// Downloads a single dataset archive (.tsv.gz) from datasets.imdbws.com to `target_path`.
/// Returns the upstream `Last-Modified` header value if present.
pub async fn download_dataset_archive(
    client: &reqwest::Client,
    filename: &str,
    target_path: &Path,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<Option<String>, String> {
    let url = format!("https://datasets.imdbws.com/{}", filename);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to request {}: {}", url, e))?;

    if !resp.status().is_success() {
        return Err(format!("Download of {} failed with HTTP status {}", filename, resp.status()));
    }

    let last_modified = resp
        .headers()
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let total_bytes = resp.content_length();

    let mut file = tokio::fs::File::create(target_path)
        .await
        .map_err(|e| format!("Failed to create temp file {}: {}", target_path.display(), e))?;

    let mut downloaded_bytes = 0u64;
    let mut last_emit = std::time::Instant::now();

    // Stream chunks into file
    let mut stream = resp;
    while let Some(chunk) = stream
        .chunk()
        .await
        .map_err(|e| format!("Error streaming chunk for {}: {}", filename, e))?
    {
        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Error writing chunk to {}: {}", target_path.display(), e))?;

        downloaded_bytes += chunk.len() as u64;

        // Emit progress throttled to at most once per 250ms
        if last_emit.elapsed().as_millis() >= 250 {
            last_emit = std::time::Instant::now();
            let mb_down = (downloaded_bytes as f64) / (1024.0 * 1024.0);
            let msg = if let Some(tot) = total_bytes {
                let mb_tot = (tot as f64) / (1024.0 * 1024.0);
                let pct = ((downloaded_bytes as f64) / (tot as f64) * 100.0) as u32;
                format!("Downloading {} ({:.1} MB / {:.1} MB, {}%)...", filename, mb_down, mb_tot, pct)
            } else {
                format!("Downloading {} ({:.1} MB)...", filename, mb_down)
            };

            if let Some(h) = app_handle {
                let _ = h.emit(
                    "imdb:index_progress",
                    IndexProgress {
                        dataset: filename.to_string(),
                        rows_processed: 0,
                        rows_indexed: 0,
                        is_finished: false,
                        message: msg,
                    },
                );
            }
        }
    }

    use tokio::io::AsyncWriteExt;
    file.flush()
        .await
        .map_err(|e| format!("Error flushing temp file {}: {}", target_path.display(), e))?;

    Ok(last_modified)
}

/// Orchestrates downloading, streaming indexing, and immediate cleanup of temporary `.tsv.gz` files.
/// All downloads strictly reside in `app_data_dir/imdb_temp/` and are deleted immediately after indexing.
pub async fn download_and_index_imdb_pipeline(
    app_data_dir: PathBuf,
    imdb_conn_arc: Arc<Mutex<Connection>>,
    app_handle: Option<tauri::AppHandle>,
) -> Result<String, String> {
    if IS_DOWNLOADING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("IMDb download or indexing is already in progress.".into());
    }
    let _guard = DownloadGuard;

    let temp_dir = app_data_dir.join("imdb_temp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temporary directory {}: {}", temp_dir.display(), e))?;

    let client = reqwest::Client::builder()
        .user_agent("PeppyMediaOrganizer/0.1.0")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let datasets = [
        ("title.basics.tsv.gz", "title.basics"),
        ("title.ratings.tsv.gz", "title.ratings"),
        ("title.episode.tsv.gz", "title.episode"),
    ];

    let mut summary = Vec::new();

    for (filename, dataset_name) in datasets {
        let temp_archive_path = temp_dir.join(filename);

        if let Some(ref h) = app_handle {
            let _ = h.emit(
                "imdb:index_progress",
                IndexProgress {
                    dataset: dataset_name.to_string(),
                    rows_processed: 0,
                    rows_indexed: 0,
                    is_finished: false,
                    message: format!("Starting download of {} from datasets.imdbws.com...", filename),
                },
            );
        }

        // 1. Download
        let last_modified = match download_dataset_archive(
            &client,
            filename,
            &temp_archive_path,
            app_handle.as_ref(),
        )
        .await
        {
            Ok(hdr) => hdr,
            Err(e) => {
                // Ensure temporary file is deleted on download error
                let _ = std::fs::remove_file(&temp_archive_path);
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Err(format!("Download failed for {}: {}", filename, e));
            }
        };

        // 2. Stream index into SQLite
        let index_result = {
            let conn_arc = imdb_conn_arc.clone();
            let app_h = app_handle.clone();
            let path_clone = temp_archive_path.clone();
            let d_name = dataset_name.to_string();

            tokio::task::spawn_blocking(move || -> Result<usize, String> {
                let mut conn = conn_arc.lock().map_err(|e| e.to_string())?;
                match d_name.as_str() {
                    "title.basics" => indexer::index_basics_tsv_gz(&mut conn, &path_clone, app_h.as_ref()),
                    "title.ratings" => indexer::index_ratings_tsv_gz(&mut conn, &path_clone, app_h.as_ref()),
                    "title.episode" => indexer::index_episodes_tsv_gz(&mut conn, &path_clone, app_h.as_ref()),
                    _ => Err(format!("Unknown dataset: {}", d_name)),
                }
            })
            .await
            .map_err(|e| format!("Join error during indexing {}: {}", dataset_name, e))?
        };

        // 3. IMMEDIATELY delete temporary archive to save disk space
        if temp_archive_path.exists() {
            let _ = std::fs::remove_file(&temp_archive_path);
        }

        let count = match index_result {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Err(format!("Indexing failed for {}: {}", dataset_name, e));
            }
        };

        // Update sync status with upstream Last-Modified header
        if let Ok(conn) = imdb_conn_arc.lock() {
            let _ = conn.execute(
                r#"
                INSERT INTO imdb_sync_status (dataset_name, last_modified_header, last_synced_at, record_count, status, error_message)
                VALUES (?1, ?2, datetime('now'), ?3, 'ready', NULL)
                ON CONFLICT(dataset_name) DO UPDATE SET
                    last_modified_header = COALESCE(excluded.last_modified_header, imdb_sync_status.last_modified_header),
                    last_synced_at = excluded.last_synced_at,
                    record_count = excluded.record_count,
                    status = excluded.status,
                    error_message = NULL
                "#,
                params![dataset_name, last_modified, count as i64],
            );
        }

        summary.push(format!("{}: {} records", dataset_name, count));
    }

    // Clean up temporary directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    let finished_msg = format!("IMDb dataset successfully updated: {}", summary.join(", "));
    if let Some(ref h) = app_handle {
        let _ = h.emit(
            "imdb:index_progress",
            IndexProgress {
                dataset: "complete".to_string(),
                rows_processed: 0,
                rows_indexed: 0,
                is_finished: true,
                message: finished_msg.clone(),
            },
        );
    }

    Ok(finished_msg)
}
