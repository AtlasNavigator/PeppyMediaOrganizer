use rusqlite::{Connection, OptionalExtension};

pub const MIGRATIONS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS library_roots (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    path            TEXT    NOT NULL UNIQUE,
    media_type      TEXT    NOT NULL CHECK(media_type IN ('movie', 'tvshow')),
    enabled         INTEGER NOT NULL DEFAULT 1,
    last_scanned_at TEXT,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS movies (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    library_root_id INTEGER NOT NULL REFERENCES library_roots(id) ON DELETE CASCADE,
    folder_path     TEXT    NOT NULL,
    file_path       TEXT    NOT NULL UNIQUE,
    title           TEXT    NOT NULL,
    sort_title      TEXT,
    year            INTEGER,
    resolution      TEXT,
    imdb_id         TEXT,
    tmdb_id         INTEGER,
    has_nfo         INTEGER NOT NULL DEFAULT 0,
    has_poster      INTEGER NOT NULL DEFAULT 0,
    has_backdrop    INTEGER NOT NULL DEFAULT 0,
    file_size       INTEGER,
    subtitle_count  INTEGER NOT NULL DEFAULT 0,
    version_label   TEXT,
    scan_status     TEXT    NOT NULL DEFAULT 'pending' CHECK(scan_status IN ('pending','matched','unmatched','error')),
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tv_shows (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    library_root_id INTEGER NOT NULL REFERENCES library_roots(id) ON DELETE CASCADE,
    folder_path     TEXT    NOT NULL UNIQUE,
    title           TEXT    NOT NULL,
    sort_title      TEXT,
    year            INTEGER,
    tvdb_id         INTEGER,
    tmdb_id         INTEGER,
    imdb_id         TEXT,
    status          TEXT,
    has_nfo         INTEGER NOT NULL DEFAULT 0,
    has_poster      INTEGER NOT NULL DEFAULT 0,
    has_backdrop    INTEGER NOT NULL DEFAULT 0,
    season_count    INTEGER NOT NULL DEFAULT 0,
    episode_count   INTEGER NOT NULL DEFAULT 0,
    scan_status     TEXT    NOT NULL DEFAULT 'pending',
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS episodes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tv_show_id      INTEGER NOT NULL REFERENCES tv_shows(id) ON DELETE CASCADE,
    file_path       TEXT    NOT NULL UNIQUE,
    title           TEXT,
    season          INTEGER NOT NULL,
    episode         INTEGER NOT NULL,
    episode_end     INTEGER,
    resolution      TEXT,
    imdb_id         TEXT,
    tvdb_id         INTEGER,
    has_nfo         INTEGER NOT NULL DEFAULT 0,
    file_size       INTEGER,
    subtitle_count  INTEGER NOT NULL DEFAULT 0,
    scan_status     TEXT    NOT NULL DEFAULT 'pending',
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS subtitles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    media_type  TEXT    NOT NULL CHECK(media_type IN ('movie', 'episode')),
    media_id    INTEGER NOT NULL,
    file_path   TEXT    NOT NULL UNIQUE,
    language    TEXT,
    flags       TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS rename_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_id        TEXT    NOT NULL,
    operation_type  TEXT    NOT NULL DEFAULT 'rename',
    media_type      TEXT    NOT NULL,
    media_id        INTEGER,
    old_path        TEXT    NOT NULL,
    new_path        TEXT    NOT NULL,
    item_type       TEXT    NOT NULL,
    executed_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    reverted_at     TEXT
);

CREATE INDEX IF NOT EXISTS idx_rename_log_batch ON rename_log(batch_id);
CREATE INDEX IF NOT EXISTS idx_movies_folder_path ON movies(folder_path);

CREATE TABLE IF NOT EXISTS app_settings (
    key         TEXT    PRIMARY KEY,
    value       TEXT    NOT NULL
);

-- Phase 2: Metadata Providers & Field Priorities
CREATE TABLE IF NOT EXISTS provider_configs (
    provider_id     TEXT PRIMARY KEY CHECK(provider_id IN ('imdb', 'tmdb', 'tvdb', 'omdb')),
    enabled         INTEGER NOT NULL DEFAULT 1,
    api_key         TEXT,
    pin_or_token    TEXT,
    base_url        TEXT,
    rate_limit_per_sec INTEGER DEFAULT 10,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS field_priorities (
    field_name      TEXT PRIMARY KEY,
    priority_order  TEXT NOT NULL -- JSON array e.g. ["imdb","tmdb","omdb"]
);

-- Phase 5: Media Streams (ffprobe technical specifications)
CREATE TABLE IF NOT EXISTS media_streams (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    media_type          TEXT    NOT NULL CHECK(media_type IN ('movie', 'episode')),
    media_id            INTEGER NOT NULL,
    file_path           TEXT    NOT NULL UNIQUE,
    container           TEXT,
    video_codec         TEXT,
    width               INTEGER,
    height              INTEGER,
    aspect_ratio        TEXT,
    hdr_format          TEXT,
    audio_codec         TEXT,
    audio_channels      INTEGER,
    audio_layout        TEXT,
    audio_languages     TEXT,
    subtitle_languages  TEXT,
    bitrate_kbps        INTEGER,
    duration_sec        REAL,
    probe_status        TEXT    NOT NULL DEFAULT 'pending' CHECK(probe_status IN ('pending', 'probed', 'failed')),
    probed_at           TEXT
);

CREATE INDEX IF NOT EXISTS idx_media_streams_media ON media_streams(media_type, media_id);
CREATE INDEX IF NOT EXISTS idx_movies_year ON movies(year);
CREATE INDEX IF NOT EXISTS idx_movies_scan_status ON movies(scan_status);
CREATE INDEX IF NOT EXISTS idx_episodes_show ON episodes(tv_show_id, season, episode);
CREATE INDEX IF NOT EXISTS idx_tv_shows_scan_status ON tv_shows(scan_status);
"#;

fn has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    col_type: &str,
) -> Result<(), rusqlite::Error> {
    if !has_column(conn, table, column)? {
        conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type),
            [],
        )?;
    }
    Ok(())
}

fn remove_episodes_composite_unique_if_present(conn: &Connection) -> Result<(), rusqlite::Error> {
    // 1. Check if table definition has composite UNIQUE on (tv_show_id, season, episode)
    let sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='episodes'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(schema) = sql {
        let lower = schema.to_ascii_lowercase();
        // If table has composite UNIQUE constraint on tv_show_id, season, episode
        if lower.contains("unique")
            && lower.contains("tv_show_id")
            && lower.contains("season")
            && lower.contains("episode")
            && (lower.contains("unique(tv_show_id, season, episode)")
                || lower.contains("unique (tv_show_id, season, episode)")
                || lower.contains("unique(tv_show_id,season,episode)"))
        {
            log::info!("Rebuilding episodes table to remove composite UNIQUE constraint on (tv_show_id, season, episode)");
            conn.execute_batch(r#"
                PRAGMA foreign_keys=OFF;
                BEGIN TRANSACTION;
                CREATE TABLE episodes_new (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    tv_show_id      INTEGER NOT NULL REFERENCES tv_shows(id) ON DELETE CASCADE,
                    file_path       TEXT    NOT NULL UNIQUE,
                    title           TEXT,
                    season          INTEGER NOT NULL,
                    episode         INTEGER NOT NULL,
                    episode_end     INTEGER,
                    resolution      TEXT,
                    imdb_id         TEXT,
                    tvdb_id         INTEGER,
                    has_nfo         INTEGER NOT NULL DEFAULT 0,
                    file_size       INTEGER,
                    subtitle_count  INTEGER NOT NULL DEFAULT 0,
                    scan_status     TEXT    NOT NULL DEFAULT 'pending',
                    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
                    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
                );
                INSERT OR IGNORE INTO episodes_new (
                    id, tv_show_id, file_path, title, season, episode, episode_end,
                    resolution, imdb_id, tvdb_id, has_nfo, file_size, subtitle_count,
                    scan_status, created_at, updated_at
                ) SELECT 
                    id, tv_show_id, file_path, title, season, episode, episode_end,
                    resolution, imdb_id, tvdb_id, has_nfo, file_size, subtitle_count,
                    scan_status, created_at, updated_at
                FROM episodes;
                DROP TABLE episodes;
                ALTER TABLE episodes_new RENAME TO episodes;
                CREATE INDEX IF NOT EXISTS idx_episodes_show ON episodes(tv_show_id, season, episode);
                COMMIT;
                PRAGMA foreign_keys=ON;
            "#)?;
        }
    }

    // 2. Check index_list for any unique index on (tv_show_id, season, episode)
    let mut stmt = conn.prepare("PRAGMA index_list('episodes')")?;
    let index_rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let unique: bool = row.get(2)?;
        Ok((name, unique))
    })?;

    let mut indices_to_drop = Vec::new();
    for row in index_rows {
        let (name, unique) = row?;
        if unique && !name.starts_with("sqlite_autoindex_episodes") {
            indices_to_drop.push(name);
        }
    }
    drop(stmt);

    for name in indices_to_drop {
        let mut info_stmt = conn.prepare(&format!("PRAGMA index_info('{}')", name))?;
        let cols: Vec<String> = info_stmt
            .query_map([], |r| r.get(2))?
            .filter_map(|r| r.ok())
            .collect();
        if cols == vec!["tv_show_id", "season", "episode"] {
            conn.execute(&format!("DROP INDEX IF EXISTS {}", name), [])?;
        }
    }

    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(MIGRATIONS_SQL)?;

    // Ensure composite unique on episodes is removed if present from prior versions
    remove_episodes_composite_unique_if_present(conn)?;

    // Safe column additions for Movies
    add_column_if_missing(conn, "movies", "plot", "TEXT")?;
    add_column_if_missing(conn, "movies", "tagline", "TEXT")?;
    add_column_if_missing(conn, "movies", "mpaa", "TEXT")?;
    add_column_if_missing(conn, "movies", "runtime", "INTEGER")?;
    add_column_if_missing(conn, "movies", "genres", "TEXT")?;
    add_column_if_missing(conn, "movies", "studio", "TEXT")?;
    add_column_if_missing(conn, "movies", "director", "TEXT")?;
    add_column_if_missing(conn, "movies", "writers", "TEXT")?;
    add_column_if_missing(conn, "movies", "actors_json", "TEXT")?;
    add_column_if_missing(conn, "movies", "poster_url", "TEXT")?;
    add_column_if_missing(conn, "movies", "backdrop_url", "TEXT")?;
    add_column_if_missing(conn, "movies", "community_rating", "REAL")?;
    add_column_if_missing(conn, "movies", "vote_count", "INTEGER")?;

    // Safe column additions for TV Shows
    add_column_if_missing(conn, "tv_shows", "plot", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "mpaa", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "studio", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "genres", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "actors_json", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "poster_url", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "backdrop_url", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "banner_url", "TEXT")?;
    add_column_if_missing(conn, "tv_shows", "episode_order_mode", "TEXT DEFAULT 'aired'")?;
    add_column_if_missing(conn, "tv_shows", "community_rating", "REAL")?;
    add_column_if_missing(conn, "tv_shows", "vote_count", "INTEGER")?;

    // Safe column additions for Episodes
    add_column_if_missing(conn, "episodes", "plot", "TEXT")?;
    add_column_if_missing(conn, "episodes", "aired_date", "TEXT")?;
    add_column_if_missing(conn, "episodes", "runtime", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "director", "TEXT")?;
    add_column_if_missing(conn, "episodes", "writer", "TEXT")?;
    add_column_if_missing(conn, "episodes", "still_url", "TEXT")?;
    add_column_if_missing(conn, "episodes", "community_rating", "REAL")?;
    add_column_if_missing(conn, "episodes", "vote_count", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "aired_season", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "aired_episode", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "dvd_season", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "dvd_episode", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "absolute_number", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "airs_before_season", "INTEGER")?;
    add_column_if_missing(conn, "episodes", "airs_before_episode", "INTEGER")?;

    // Safe column additions for rename_log
    add_column_if_missing(conn, "rename_log", "operation_type", "TEXT DEFAULT 'rename'")?;
    add_column_if_missing(conn, "rename_log", "media_type", "TEXT DEFAULT 'movie'")?;
    add_column_if_missing(conn, "rename_log", "media_id", "INTEGER")?;
    add_column_if_missing(conn, "rename_log", "item_type", "TEXT DEFAULT 'video'")?;
    add_column_if_missing(conn, "rename_log", "reverted_at", "TEXT")?;

    // Default Provider Configurations
    conn.execute_batch(r#"
        INSERT OR IGNORE INTO provider_configs (provider_id, enabled, rate_limit_per_sec) VALUES
            ('imdb', 1, 100),
            ('tmdb', 1, 40),
            ('tvdb', 1, 20),
            ('omdb', 1, 10);

        INSERT OR IGNORE INTO field_priorities (field_name, priority_order) VALUES
            ('title', '["tmdb", "imdb", "tvdb", "omdb"]'),
            ('overview', '["tmdb", "tvdb", "omdb", "imdb"]'),
            ('tagline', '["tmdb", "omdb"]'),
            ('ratings', '["imdb", "tmdb", "omdb"]'),
            ('genres', '["tmdb", "imdb", "tvdb", "omdb"]'),
            ('studio', '["tmdb", "tvdb"]'),
            ('cast', '["tmdb", "imdb", "tvdb"]'),
            ('crew', '["tmdb", "imdb", "tvdb"]'),
            ('posters', '["tmdb", "tvdb"]'),
            ('backdrops', '["tmdb", "tvdb"]'),
            ('banners', '["tvdb", "tmdb"]'),
            ('ids', '["imdb", "tmdb", "tvdb"]'),
            ('certification', '["tmdb", "omdb", "imdb"]');
    "#)?;

    Ok(())
}
