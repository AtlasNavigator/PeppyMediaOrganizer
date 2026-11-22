use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use flate2::read::GzDecoder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexProgress {
    pub dataset: String,
    pub rows_processed: usize,
    pub rows_indexed: usize,
    pub is_finished: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetUpdateInfo {
    pub dataset_name: String,
    pub upstream_last_modified: String,
    pub local_last_synced: Option<String>,
    pub update_available: bool,
}

/// Helper function to parse an IMDb TSV column, treating "\N" as None.
fn parse_tsv_val(s: &str) -> Option<&str> {
    if s == r"\N" || s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Streams and indexes `title.basics.tsv.gz` into SQLite in batches.
pub fn index_basics_tsv_gz(
    conn: &mut Connection,
    path: &Path,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<usize, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let gz = GzDecoder::new(file);
    let reader = BufReader::with_capacity(512 * 1024, gz);

    let mut rows_processed = 0usize;
    let mut rows_indexed = 0usize;

    // Filter list of relevant title types
    let target_types = ["movie", "tvSeries", "tvMiniSeries", "tvMovie", "tvSpecial"];

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare(
                r#"
                INSERT OR REPLACE INTO imdb_titles (
                    tconst, title_type, primary_title, original_title,
                    is_adult, start_year, end_year, runtime_minutes, genres
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )
            .map_err(|e| e.to_string())?;

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => continue,
            };

            rows_processed += 1;
            if rows_processed == 1 {
                continue; // Skip TSV header
            }

            let mut parts = line.split('\t');
            let tconst = match parts.next() {
                Some(v) => v,
                None => continue,
            };
            let title_type = match parts.next() {
                Some(v) => v,
                None => continue,
            };
            if !target_types.contains(&title_type) {
                continue;
            }

            let primary_title = match parts.next() {
                Some(v) => v,
                None => continue,
            };
            let original_title = parse_tsv_val(match parts.next() {
                Some(v) => v,
                None => continue,
            });
            let is_adult = match parts.next() {
                Some(v) => v.parse::<i32>().unwrap_or(0),
                None => 0,
            };
            let start_year = parse_tsv_val(match parts.next() {
                Some(v) => v,
                None => continue,
            })
            .and_then(|y| y.parse::<i32>().ok());
            let end_year = parse_tsv_val(match parts.next() {
                Some(v) => v,
                None => continue,
            })
            .and_then(|y| y.parse::<i32>().ok());
            let runtime_minutes = parse_tsv_val(match parts.next() {
                Some(v) => v,
                None => continue,
            })
            .and_then(|m| m.parse::<i32>().ok());
            let genres = parse_tsv_val(match parts.next() {
                Some(v) => v,
                None => continue,
            });

            let _ = stmt.execute(params![
                tconst,
                title_type,
                primary_title,
                original_title,
                is_adult,
                start_year,
                end_year,
                runtime_minutes,
                genres
            ]);

            rows_indexed += 1;

            if rows_processed % 250_000 == 0 {
                if let Some(h) = app_handle {
                    let _ = h.emit(
                        "imdb:index_progress",
                        IndexProgress {
                            dataset: "title.basics".to_string(),
                            rows_processed,
                            rows_indexed,
                            is_finished: false,
                            message: format!("Indexed {} movie/series titles...", rows_indexed),
                        },
                    );
                }
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    // Record sync status
    let _ = conn.execute(
        r#"
        INSERT OR REPLACE INTO imdb_sync_status (dataset_name, last_synced_at, record_count, status)
        VALUES ('title.basics', datetime('now'), ?1, 'ready')
        "#,
        params![rows_indexed as i64],
    );

    if let Some(h) = app_handle {
        let _ = h.emit(
            "imdb:index_progress",
            IndexProgress {
                dataset: "title.basics".to_string(),
                rows_processed,
                rows_indexed,
                is_finished: true,
                message: format!("Finished! Total {} titles indexed.", rows_indexed),
            },
        );
    }

    Ok(rows_indexed)
}

/// Streams and indexes `title.ratings.tsv.gz`.
pub fn index_ratings_tsv_gz(
    conn: &mut Connection,
    path: &Path,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<usize, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let gz = GzDecoder::new(file);
    let reader = BufReader::with_capacity(256 * 1024, gz);

    let mut rows_processed = 0usize;
    let mut rows_indexed = 0usize;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare(
                r#"
                INSERT OR REPLACE INTO imdb_ratings (tconst, average_rating, num_votes)
                VALUES (?1, ?2, ?3)
                "#,
            )
            .map_err(|e| e.to_string())?;

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => continue,
            };

            rows_processed += 1;
            if rows_processed == 1 {
                continue; // Skip header
            }

            let mut parts = line.split('\t');
            let tconst = match parts.next() {
                Some(v) => v,
                None => continue,
            };
            let rating = match parts.next().and_then(|r| r.parse::<f64>().ok()) {
                Some(r) => r,
                None => continue,
            };
            let votes = match parts.next().and_then(|v| v.parse::<i64>().ok()) {
                Some(v) => v,
                None => continue,
            };

            let _ = stmt.execute(params![tconst, rating, votes]);
            rows_indexed += 1;

            if rows_processed % 250_000 == 0 {
                if let Some(h) = app_handle {
                    let _ = h.emit(
                        "imdb:index_progress",
                        IndexProgress {
                            dataset: "title.ratings".to_string(),
                            rows_processed,
                            rows_indexed,
                            is_finished: false,
                            message: format!("Indexed {} ratings...", rows_indexed),
                        },
                    );
                }
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    let _ = conn.execute(
        r#"
        INSERT OR REPLACE INTO imdb_sync_status (dataset_name, last_synced_at, record_count, status)
        VALUES ('title.ratings', datetime('now'), ?1, 'ready')
        "#,
        params![rows_indexed as i64],
    );

    if let Some(h) = app_handle {
        let _ = h.emit(
            "imdb:index_progress",
            IndexProgress {
                dataset: "title.ratings".to_string(),
                rows_processed,
                rows_indexed,
                is_finished: true,
                message: format!("Finished ratings! Total: {}", rows_indexed),
            },
        );
    }

    Ok(rows_indexed)
}

/// Streams and indexes `title.episode.tsv.gz`.
pub fn index_episodes_tsv_gz(
    conn: &mut Connection,
    path: &Path,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<usize, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let gz = GzDecoder::new(file);
    let reader = BufReader::with_capacity(256 * 1024, gz);

    let mut rows_processed = 0usize;
    let mut rows_indexed = 0usize;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare(
                r#"
                INSERT OR REPLACE INTO imdb_episodes (tconst, parent_tconst, season_number, episode_number)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )
            .map_err(|e| e.to_string())?;

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => continue,
            };

            rows_processed += 1;
            if rows_processed == 1 {
                continue; // Skip header
            }

            let mut parts = line.split('\t');
            let tconst = match parts.next() {
                Some(v) => v,
                None => continue,
            };
            let parent_tconst = match parts.next() {
                Some(v) => v,
                None => continue,
            };
            let season = parse_tsv_val(match parts.next() {
                Some(v) => v,
                None => continue,
            })
            .and_then(|s| s.parse::<i32>().ok());
            let episode = parse_tsv_val(match parts.next() {
                Some(v) => v,
                None => continue,
            })
            .and_then(|e| e.parse::<i32>().ok());

            let _ = stmt.execute(params![tconst, parent_tconst, season, episode]);
            rows_indexed += 1;

            if rows_processed % 500_000 == 0 {
                if let Some(h) = app_handle {
                    let _ = h.emit(
                        "imdb:index_progress",
                        IndexProgress {
                            dataset: "title.episode".to_string(),
                            rows_processed,
                            rows_indexed,
                            is_finished: false,
                            message: format!("Indexed {} episode mappings...", rows_indexed),
                        },
                    );
                }
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())?;

    let _ = conn.execute(
        r#"
        INSERT OR REPLACE INTO imdb_sync_status (dataset_name, last_synced_at, record_count, status)
        VALUES ('title.episode', datetime('now'), ?1, 'ready')
        "#,
        params![rows_indexed as i64],
    );

    if let Some(h) = app_handle {
        let _ = h.emit(
            "imdb:index_progress",
            IndexProgress {
                dataset: "title.episode".to_string(),
                rows_processed,
                rows_indexed,
                is_finished: true,
                message: format!("Finished episodes! Total: {}", rows_indexed),
            },
        );
    }

    Ok(rows_indexed)
}

/// Checks datasets.imdbws.com for dataset Last-Modified headers via HTTP HEAD.
pub async fn check_imdb_upstream_updates(
    local_headers: &std::collections::HashMap<String, (Option<String>, Option<String>)>,
) -> Result<Vec<DatasetUpdateInfo>, String> {
    let client = reqwest::Client::builder()
        .user_agent("PeppyMediaOrganizer/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let datasets = ["title.basics.tsv.gz", "title.ratings.tsv.gz", "title.episode.tsv.gz"];
    let mut infos = Vec::new();

    for filename in datasets {
        let url = format!("https://datasets.imdbws.com/{}", filename);
        let resp = client.head(&url).send().await.map_err(|e| e.to_string())?;

        let last_modified = resp
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();

        let dataset_name = filename.trim_end_matches(".tsv.gz").to_string();

        let (local_synced, local_header) = local_headers
            .get(&dataset_name)
            .cloned()
            .unwrap_or((None, None));

        let update_available = match &local_header {
            Some(h) => h != &last_modified,
            None => true,
        };

        infos.push(DatasetUpdateInfo {
            dataset_name,
            upstream_last_modified: last_modified,
            local_last_synced: local_synced,
            update_available,
        });
    }

    Ok(infos)
}
