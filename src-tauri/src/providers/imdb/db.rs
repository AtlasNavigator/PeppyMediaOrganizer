use std::path::Path;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImdbSearchResult {
    pub tconst: String,
    pub title_type: String,
    pub primary_title: String,
    pub original_title: Option<String>,
    pub start_year: Option<u16>,
    pub runtime_minutes: Option<u32>,
    pub genres: Vec<String>,
    pub rating: Option<f64>,
    pub num_votes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImdbSyncStatus {
    pub dataset_name: String,
    pub last_modified_header: Option<String>,
    pub last_synced_at: Option<String>,
    pub record_count: usize,
    pub status: String,
    pub error_message: Option<String>,
}

pub fn init_imdb_db(db_path: &Path) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;

        CREATE TABLE IF NOT EXISTS imdb_titles (
            tconst          TEXT PRIMARY KEY,
            title_type      TEXT NOT NULL,
            primary_title   TEXT NOT NULL,
            original_title  TEXT,
            is_adult        INTEGER NOT NULL DEFAULT 0,
            start_year      INTEGER,
            end_year        INTEGER,
            runtime_minutes INTEGER,
            genres          TEXT
        );

        CREATE TABLE IF NOT EXISTS imdb_ratings (
            tconst          TEXT PRIMARY KEY,
            average_rating  REAL NOT NULL,
            num_votes       INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS imdb_episodes (
            tconst          TEXT PRIMARY KEY,
            parent_tconst   TEXT NOT NULL,
            season_number   INTEGER,
            episode_number  INTEGER
        );

        CREATE TABLE IF NOT EXISTS imdb_sync_status (
            dataset_name        TEXT PRIMARY KEY,
            last_modified_header TEXT,
            last_synced_at      TEXT,
            record_count        INTEGER NOT NULL DEFAULT 0,
            status              TEXT NOT NULL DEFAULT 'idle',
            error_message       TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_imdb_title_search ON imdb_titles(primary_title COLLATE NOCASE, start_year);
        CREATE INDEX IF NOT EXISTS idx_imdb_episodes_parent ON imdb_episodes(parent_tconst, season_number, episode_number);
        "#,
    )?;
    Ok(conn)
}

pub fn get_sync_status(conn: &Connection) -> Result<Vec<ImdbSyncStatus>> {
    let mut stmt = conn.prepare(
        "SELECT dataset_name, last_modified_header, last_synced_at, record_count, status, error_message FROM imdb_sync_status",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ImdbSyncStatus {
            dataset_name: row.get(0)?,
            last_modified_header: row.get(1)?,
            last_synced_at: row.get(2)?,
            record_count: row.get::<_, i64>(3)? as usize,
            status: row.get(4)?,
            error_message: row.get(5)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

pub fn get_title_by_tconst(conn: &Connection, tconst: &str) -> Result<Option<ImdbSearchResult>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT t.tconst, t.title_type, t.primary_title, t.original_title,
               t.start_year, t.runtime_minutes, t.genres,
               r.average_rating, r.num_votes
        FROM imdb_titles t
        LEFT JOIN imdb_ratings r ON t.tconst = r.tconst
        WHERE t.tconst = ?1
        "#,
    )?;

    let mut rows = stmt.query_map(params![tconst], |row| {
        let genres_raw: Option<String> = row.get(6)?;
        let genres = genres_raw
            .map(|g| g.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        Ok(ImdbSearchResult {
            tconst: row.get(0)?,
            title_type: row.get(1)?,
            primary_title: row.get(2)?,
            original_title: row.get(3)?,
            start_year: row.get::<_, Option<i32>>(4)?.map(|y| y as u16),
            runtime_minutes: row.get::<_, Option<i32>>(5)?.map(|m| m as u32),
            genres,
            rating: row.get(7)?,
            num_votes: row.get::<_, Option<i32>>(8)?.map(|v| v as u32),
        })
    })?;

    if let Some(r) = rows.next() {
        Ok(Some(r?))
    } else {
        Ok(None)
    }
}

pub fn search_titles(
    conn: &Connection,
    title_query: &str,
    year_filter: Option<u16>,
    media_type: Option<&str>,
    limit: usize,
) -> Result<Vec<ImdbSearchResult>> {
    let clean_query = title_query.trim();

    // If query matches tt-ID directly:
    if clean_query.starts_with("tt") && clean_query[2..].chars().all(|c| c.is_ascii_digit()) {
        if let Some(result) = get_title_by_tconst(conn, clean_query)? {
            return Ok(vec![result]);
        }
    }

    let query_param = format!("%{}%", clean_query);

    let type_filter = match media_type {
        Some("movie") => "AND t.title_type IN ('movie', 'tvMovie')",
        Some("tvshow") => "AND t.title_type IN ('tvSeries', 'tvMiniSeries')",
        _ => "",
    };

    let sql = format!(
        r#"
        SELECT t.tconst, t.title_type, t.primary_title, t.original_title,
               t.start_year, t.runtime_minutes, t.genres,
               r.average_rating, r.num_votes
        FROM imdb_titles t
        LEFT JOIN imdb_ratings r ON t.tconst = r.tconst
        WHERE (t.primary_title LIKE ?1 OR t.original_title LIKE ?1)
          {}
          AND (?2 IS NULL OR t.start_year = ?2)
        ORDER BY COALESCE(r.num_votes, 0) DESC, t.start_year DESC
        LIMIT ?3
        "#,
        type_filter
    );

    let mut stmt = conn.prepare(&sql)?;
    let year_param = year_filter.map(|y| y as i32);

    let rows = stmt.query_map(params![query_param, year_param, limit as i64], |row| {
        let genres_raw: Option<String> = row.get(6)?;
        let genres = genres_raw
            .map(|g| g.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        Ok(ImdbSearchResult {
            tconst: row.get(0)?,
            title_type: row.get(1)?,
            primary_title: row.get(2)?,
            original_title: row.get(3)?,
            start_year: row.get::<_, Option<i32>>(4)?.map(|y| y as u16),
            runtime_minutes: row.get::<_, Option<i32>>(5)?.map(|m| m as u32),
            genres,
            rating: row.get(7)?,
            num_votes: row.get::<_, Option<i32>>(8)?.map(|v| v as u32),
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
