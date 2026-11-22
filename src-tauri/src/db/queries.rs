use rusqlite::{params, Connection, Result};
use crate::db::models::{
    DuplicateEpisodeItem, DuplicateMovieItem, Episode, EpisodeDuplicateGroup, LibraryRoot,
    MediaStreamInfo, Movie, MovieDuplicateGroup, TvShow,
};

pub fn insert_library_root(conn: &Connection, path: &str, media_type: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO library_roots (path, media_type) VALUES (?1, ?2)",
        params![path, media_type],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_library_roots(conn: &Connection) -> Result<Vec<LibraryRoot>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, media_type, enabled, last_scanned_at, created_at FROM library_roots ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(LibraryRoot {
            id: row.get(0)?,
            path: row.get(1)?,
            media_type: row.get(2)?,
            enabled: row.get(3)?,
            last_scanned_at: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    let mut roots = Vec::new();
    for root in rows {
        roots.push(root?);
    }
    Ok(roots)
}

pub fn get_library_root_by_id(conn: &Connection, id: i64) -> Result<Option<LibraryRoot>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, media_type, enabled, last_scanned_at, created_at FROM library_roots WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(LibraryRoot {
            id: row.get(0)?,
            path: row.get(1)?,
            media_type: row.get(2)?,
            enabled: row.get(3)?,
            last_scanned_at: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    if let Some(root) = rows.next() {
        Ok(Some(root?))
    } else {
        Ok(None)
    }
}

pub fn delete_library_root(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM library_roots WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn toggle_library_root(conn: &Connection, id: i64, enabled: bool) -> Result<()> {
    let enabled_val = if enabled { 1 } else { 0 };
    conn.execute(
        "UPDATE library_roots SET enabled = ?1 WHERE id = ?2",
        params![enabled_val, id],
    )?;
    Ok(())
}

pub fn update_library_root_scan_time(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE library_roots SET last_scanned_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn clear_library_data(conn: &Connection, library_root_id: i64) -> Result<()> {
    conn.execute("DELETE FROM movies WHERE library_root_id = ?1", params![library_root_id])?;
    conn.execute("DELETE FROM tv_shows WHERE library_root_id = ?1", params![library_root_id])?;
    Ok(())
}

pub fn insert_movie(conn: &Connection, movie: &Movie) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO movies (
            library_root_id, folder_path, file_path, title, sort_title, year,
            resolution, imdb_id, tmdb_id, has_nfo, has_poster, has_backdrop,
            file_size, subtitle_count, version_label, scan_status, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, datetime('now'))
        ON CONFLICT(file_path) DO UPDATE SET
            library_root_id = excluded.library_root_id,
            folder_path = excluded.folder_path,
            title = excluded.title,
            sort_title = excluded.sort_title,
            year = excluded.year,
            resolution = excluded.resolution,
            imdb_id = excluded.imdb_id,
            tmdb_id = excluded.tmdb_id,
            has_nfo = excluded.has_nfo,
            has_poster = excluded.has_poster,
            has_backdrop = excluded.has_backdrop,
            file_size = excluded.file_size,
            subtitle_count = excluded.subtitle_count,
            version_label = excluded.version_label,
            scan_status = excluded.scan_status,
            updated_at = datetime('now')
        "#,
        params![
            movie.library_root_id,
            movie.folder_path,
            movie.file_path,
            movie.title,
            movie.sort_title,
            movie.year,
            movie.resolution,
            movie.imdb_id,
            movie.tmdb_id,
            movie.has_nfo,
            movie.has_poster,
            movie.has_backdrop,
            movie.file_size,
            movie.subtitle_count,
            movie.version_label,
            movie.scan_status,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_movies(conn: &Connection, library_root_id: Option<i64>) -> Result<Vec<Movie>> {
    let query = match library_root_id {
        Some(_) => "SELECT id, library_root_id, folder_path, file_path, title, sort_title, year, resolution, imdb_id, tmdb_id, has_nfo, has_poster, has_backdrop, file_size, subtitle_count, version_label, scan_status, created_at, updated_at FROM movies WHERE library_root_id = ?1 ORDER BY title COLLATE NOCASE ASC",
        None => "SELECT id, library_root_id, folder_path, file_path, title, sort_title, year, resolution, imdb_id, tmdb_id, has_nfo, has_poster, has_backdrop, file_size, subtitle_count, version_label, scan_status, created_at, updated_at FROM movies ORDER BY title COLLATE NOCASE ASC",
    };

    let mut stmt = conn.prepare(query)?;
    let row_mapper = |row: &rusqlite::Row| {
        Ok(Movie {
            id: row.get(0)?,
            library_root_id: row.get(1)?,
            folder_path: row.get(2)?,
            file_path: row.get(3)?,
            title: row.get(4)?,
            sort_title: row.get(5)?,
            year: row.get(6)?,
            resolution: row.get(7)?,
            imdb_id: row.get(8)?,
            tmdb_id: row.get(9)?,
            has_nfo: row.get(10)?,
            has_poster: row.get(11)?,
            has_backdrop: row.get(12)?,
            file_size: row.get(13)?,
            subtitle_count: row.get(14)?,
            version_label: row.get(15)?,
            scan_status: row.get(16)?,
            created_at: row.get(17)?,
            updated_at: row.get(18)?,
        })
    };

    let mut movies = Vec::new();
    if let Some(root_id) = library_root_id {
        let rows = stmt.query_map(params![root_id], row_mapper)?;
        for movie in rows {
            movies.push(movie?);
        }
    } else {
        let rows = stmt.query_map([], row_mapper)?;
        for movie in rows {
            movies.push(movie?);
        }
    }
    Ok(movies)
}

pub fn get_movie_by_id(conn: &Connection, id: i64) -> Result<Option<Movie>> {
    let mut stmt = conn.prepare(
        "SELECT id, library_root_id, folder_path, file_path, title, sort_title, year, resolution, imdb_id, tmdb_id, has_nfo, has_poster, has_backdrop, file_size, subtitle_count, version_label, scan_status, created_at, updated_at FROM movies WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Movie {
            id: row.get(0)?,
            library_root_id: row.get(1)?,
            folder_path: row.get(2)?,
            file_path: row.get(3)?,
            title: row.get(4)?,
            sort_title: row.get(5)?,
            year: row.get(6)?,
            resolution: row.get(7)?,
            imdb_id: row.get(8)?,
            tmdb_id: row.get(9)?,
            has_nfo: row.get(10)?,
            has_poster: row.get(11)?,
            has_backdrop: row.get(12)?,
            file_size: row.get(13)?,
            subtitle_count: row.get(14)?,
            version_label: row.get(15)?,
            scan_status: row.get(16)?,
            created_at: row.get(17)?,
            updated_at: row.get(18)?,
        })
    })?;

    if let Some(movie) = rows.next() {
        Ok(Some(movie?))
    } else {
        Ok(None)
    }
}

pub fn insert_tv_show(conn: &Connection, show: &TvShow) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO tv_shows (
            library_root_id, folder_path, title, sort_title, year,
            tvdb_id, tmdb_id, imdb_id, status, has_nfo, has_poster, has_backdrop,
            season_count, episode_count, scan_status, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, datetime('now'))
        ON CONFLICT(folder_path) DO UPDATE SET
            library_root_id = excluded.library_root_id,
            title = excluded.title,
            sort_title = excluded.sort_title,
            year = excluded.year,
            tvdb_id = excluded.tvdb_id,
            tmdb_id = excluded.tmdb_id,
            imdb_id = excluded.imdb_id,
            status = excluded.status,
            has_nfo = excluded.has_nfo,
            has_poster = excluded.has_poster,
            has_backdrop = excluded.has_backdrop,
            season_count = excluded.season_count,
            episode_count = excluded.episode_count,
            scan_status = excluded.scan_status,
            updated_at = datetime('now')
        "#,
        params![
            show.library_root_id,
            show.folder_path,
            show.title,
            show.sort_title,
            show.year,
            show.tvdb_id,
            show.tmdb_id,
            show.imdb_id,
            show.status,
            show.has_nfo,
            show.has_poster,
            show.has_backdrop,
            show.season_count,
            show.episode_count,
            show.scan_status,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_tv_shows(conn: &Connection, library_root_id: Option<i64>) -> Result<Vec<TvShow>> {
    let query = match library_root_id {
        Some(_) => "SELECT id, library_root_id, folder_path, title, sort_title, year, tvdb_id, tmdb_id, imdb_id, status, has_nfo, has_poster, has_backdrop, season_count, episode_count, scan_status, created_at, updated_at FROM tv_shows WHERE library_root_id = ?1 ORDER BY title COLLATE NOCASE ASC",
        None => "SELECT id, library_root_id, folder_path, title, sort_title, year, tvdb_id, tmdb_id, imdb_id, status, has_nfo, has_poster, has_backdrop, season_count, episode_count, scan_status, created_at, updated_at FROM tv_shows ORDER BY title COLLATE NOCASE ASC",
    };

    let mut stmt = conn.prepare(query)?;
    let row_mapper = |row: &rusqlite::Row| {
        Ok(TvShow {
            id: row.get(0)?,
            library_root_id: row.get(1)?,
            folder_path: row.get(2)?,
            title: row.get(3)?,
            sort_title: row.get(4)?,
            year: row.get(5)?,
            tvdb_id: row.get(6)?,
            tmdb_id: row.get(7)?,
            imdb_id: row.get(8)?,
            status: row.get(9)?,
            has_nfo: row.get(10)?,
            has_poster: row.get(11)?,
            has_backdrop: row.get(12)?,
            season_count: row.get(13)?,
            episode_count: row.get(14)?,
            scan_status: row.get(15)?,
            created_at: row.get(16)?,
            updated_at: row.get(17)?,
        })
    };

    let mut shows = Vec::new();
    if let Some(root_id) = library_root_id {
        let rows = stmt.query_map(params![root_id], row_mapper)?;
        for show in rows {
            shows.push(show?);
        }
    } else {
        let rows = stmt.query_map([], row_mapper)?;
        for show in rows {
            shows.push(show?);
        }
    }
    Ok(shows)
}

pub fn get_tv_show_by_id(conn: &Connection, id: i64) -> Result<Option<TvShow>> {
    let mut stmt = conn.prepare(
        "SELECT id, library_root_id, folder_path, title, sort_title, year, tvdb_id, tmdb_id, imdb_id, status, has_nfo, has_poster, has_backdrop, season_count, episode_count, scan_status, created_at, updated_at FROM tv_shows WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(TvShow {
            id: row.get(0)?,
            library_root_id: row.get(1)?,
            folder_path: row.get(2)?,
            title: row.get(3)?,
            sort_title: row.get(4)?,
            year: row.get(5)?,
            tvdb_id: row.get(6)?,
            tmdb_id: row.get(7)?,
            imdb_id: row.get(8)?,
            status: row.get(9)?,
            has_nfo: row.get(10)?,
            has_poster: row.get(11)?,
            has_backdrop: row.get(12)?,
            season_count: row.get(13)?,
            episode_count: row.get(14)?,
            scan_status: row.get(15)?,
            created_at: row.get(16)?,
            updated_at: row.get(17)?,
        })
    })?;

    if let Some(show) = rows.next() {
        Ok(Some(show?))
    } else {
        Ok(None)
    }
}

pub fn insert_episode(conn: &Connection, episode: &Episode) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO episodes (
            tv_show_id, file_path, title, season, episode, episode_end,
            resolution, imdb_id, tvdb_id, has_nfo, file_size, subtitle_count,
            scan_status, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))
        ON CONFLICT(file_path) DO UPDATE SET
            tv_show_id = excluded.tv_show_id,
            title = excluded.title,
            season = excluded.season,
            episode = excluded.episode,
            episode_end = excluded.episode_end,
            resolution = excluded.resolution,
            imdb_id = excluded.imdb_id,
            tvdb_id = excluded.tvdb_id,
            has_nfo = excluded.has_nfo,
            file_size = excluded.file_size,
            subtitle_count = excluded.subtitle_count,
            scan_status = excluded.scan_status,
            updated_at = datetime('now')
        "#,
        params![
            episode.tv_show_id,
            episode.file_path,
            episode.title,
            episode.season,
            episode.episode,
            episode.episode_end,
            episode.resolution,
            episode.imdb_id,
            episode.tvdb_id,
            episode.has_nfo,
            episode.file_size,
            episode.subtitle_count,
            episode.scan_status,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_episodes_for_show(conn: &Connection, tv_show_id: i64) -> Result<Vec<Episode>> {
    let mut stmt = conn.prepare(
        "SELECT id, tv_show_id, file_path, title, season, episode, episode_end, resolution, imdb_id, tvdb_id, has_nfo, file_size, subtitle_count, scan_status, created_at, updated_at FROM episodes WHERE tv_show_id = ?1 ORDER BY season ASC, episode ASC",
    )?;
    let rows = stmt.query_map(params![tv_show_id], |row| {
        Ok(Episode {
            id: row.get(0)?,
            tv_show_id: row.get(1)?,
            file_path: row.get(2)?,
            title: row.get(3)?,
            season: row.get(4)?,
            episode: row.get(5)?,
            episode_end: row.get(6)?,
            resolution: row.get(7)?,
            imdb_id: row.get(8)?,
            tvdb_id: row.get(9)?,
            has_nfo: row.get(10)?,
            file_size: row.get(11)?,
            subtitle_count: row.get(12)?,
            scan_status: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    })?;

    let mut eps = Vec::new();
    for ep in rows {
        eps.push(ep?);
    }
    Ok(eps)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
    if let Some(val) = rows.next() {
        Ok(Some(val?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 5: Media Streams (ffprobe technical specifications)
// ---------------------------------------------------------------------------

pub fn upsert_media_stream(conn: &Connection, stream: &MediaStreamInfo) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO media_streams (
            media_type, media_id, file_path, container, video_codec,
            width, height, aspect_ratio, hdr_format, audio_codec,
            audio_channels, audio_layout, audio_languages, subtitle_languages,
            bitrate_kbps, duration_sec, probe_status, probed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, datetime('now'))
        ON CONFLICT(file_path) DO UPDATE SET
            media_type = excluded.media_type,
            media_id = excluded.media_id,
            container = excluded.container,
            video_codec = excluded.video_codec,
            width = excluded.width,
            height = excluded.height,
            aspect_ratio = excluded.aspect_ratio,
            hdr_format = excluded.hdr_format,
            audio_codec = excluded.audio_codec,
            audio_channels = excluded.audio_channels,
            audio_layout = excluded.audio_layout,
            audio_languages = excluded.audio_languages,
            subtitle_languages = excluded.subtitle_languages,
            bitrate_kbps = excluded.bitrate_kbps,
            duration_sec = excluded.duration_sec,
            probe_status = excluded.probe_status,
            probed_at = datetime('now')
        "#,
        params![
            stream.media_type,
            stream.media_id,
            stream.file_path,
            stream.container,
            stream.video_codec,
            stream.width,
            stream.height,
            stream.aspect_ratio,
            stream.hdr_format,
            stream.audio_codec,
            stream.audio_channels,
            stream.audio_layout,
            stream.audio_languages,
            stream.subtitle_languages,
            stream.bitrate_kbps,
            stream.duration_sec,
            stream.probe_status,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_media_stream_by_media(
    conn: &Connection,
    media_type: &str,
    media_id: i64,
) -> Result<Option<MediaStreamInfo>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, media_type, media_id, file_path, container, video_codec,
               width, height, aspect_ratio, hdr_format, audio_codec,
               audio_channels, audio_layout, audio_languages, subtitle_languages,
               bitrate_kbps, duration_sec, probe_status, probed_at
        FROM media_streams
        WHERE media_type = ?1 AND media_id = ?2
        LIMIT 1
        "#,
    )?;

    let mut rows = stmt.query_map(params![media_type, media_id], |row| {
        Ok(MediaStreamInfo {
            id: row.get(0)?,
            media_type: row.get(1)?,
            media_id: row.get(2)?,
            file_path: row.get(3)?,
            container: row.get(4)?,
            video_codec: row.get(5)?,
            width: row.get(6)?,
            height: row.get(7)?,
            aspect_ratio: row.get(8)?,
            hdr_format: row.get(9)?,
            audio_codec: row.get(10)?,
            audio_channels: row.get(11)?,
            audio_layout: row.get(12)?,
            audio_languages: row.get(13)?,
            subtitle_languages: row.get(14)?,
            bitrate_kbps: row.get(15)?,
            duration_sec: row.get(16)?,
            probe_status: row.get(17)?,
            probed_at: row.get(18)?,
        })
    })?;

    if let Some(stream) = rows.next() {
        Ok(Some(stream?))
    } else {
        Ok(None)
    }
}

pub fn get_pending_probe_paths(conn: &Connection, limit: usize) -> Result<Vec<(String, String, i64)>> {
    // 1. Pending movies without probe info
    let mut stmt = conn.prepare(
        r#"
        SELECT m.file_path, 'movie', m.id
        FROM movies m
        LEFT JOIN media_streams ms ON ms.media_type = 'movie' AND ms.media_id = m.id
        WHERE ms.id IS NULL OR ms.probe_status = 'pending'
        LIMIT ?1
        "#,
    )?;

    let mut results = Vec::new();
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    for r in rows {
        results.push(r?);
    }

    // 2. Pending episodes if budget remains
    let remaining = limit.saturating_sub(results.len());
    if remaining > 0 {
        let mut ep_stmt = conn.prepare(
            r#"
            SELECT e.file_path, 'episode', e.id
            FROM episodes e
            LEFT JOIN media_streams ms ON ms.media_type = 'episode' AND ms.media_id = e.id
            WHERE ms.id IS NULL OR ms.probe_status = 'pending'
            LIMIT ?1
            "#,
        )?;
        let ep_rows = ep_stmt.query_map(params![remaining as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        for r in ep_rows {
            results.push(r?);
        }
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Phase 5: Duplicate Detection Queries
// ---------------------------------------------------------------------------

pub fn find_movie_duplicates(conn: &Connection) -> Result<Vec<MovieDuplicateGroup>> {
    let mut groups: Vec<MovieDuplicateGroup> = Vec::new();

    // 1. Group by IMDb ID
    {
        let mut imdb_stmt = conn.prepare(
            r#"
            SELECT imdb_id
            FROM movies
            WHERE imdb_id IS NOT NULL AND imdb_id != ''
            GROUP BY imdb_id
            HAVING count(*) > 1
            "#,
        )?;
        let imdb_ids: Vec<String> = imdb_stmt
            .query_map([], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        for id in imdb_ids {
            let mut item_stmt = conn.prepare(
                r#"
                SELECT m.id, m.library_root_id, m.folder_path, m.file_path, m.title, m.year,
                       m.resolution, m.imdb_id, m.tmdb_id, m.file_size, m.version_label,
                       ms.video_codec, ms.audio_codec, ms.bitrate_kbps
                FROM movies m
                LEFT JOIN media_streams ms ON ms.media_type = 'movie' AND ms.media_id = m.id
                WHERE m.imdb_id = ?1
                ORDER BY m.file_size DESC
                "#,
            )?;
            let items: Vec<DuplicateMovieItem> = item_stmt
                .query_map(params![id], |row| {
                    Ok(DuplicateMovieItem {
                        id: row.get(0)?,
                        library_root_id: row.get(1)?,
                        folder_path: row.get(2)?,
                        file_path: row.get(3)?,
                        title: row.get(4)?,
                        year: row.get(5)?,
                        resolution: row.get(6)?,
                        imdb_id: row.get(7)?,
                        tmdb_id: row.get(8)?,
                        file_size: row.get(9)?,
                        version_label: row.get(10)?,
                        video_codec: row.get(11)?,
                        audio_codec: row.get(12)?,
                        bitrate_kbps: row.get(13)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            if items.len() > 1 {
                groups.push(MovieDuplicateGroup {
                    match_key: format!("imdb:{}", id),
                    reason: format!("Identical IMDb ID ({})", id),
                    items,
                });
            }
        }
    }

    // 2. Group by TMDb ID (where IMDb was not matched)
    {
        let mut tmdb_stmt = conn.prepare(
            r#"
            SELECT tmdb_id
            FROM movies
            WHERE tmdb_id IS NOT NULL AND (imdb_id IS NULL OR imdb_id = '')
            GROUP BY tmdb_id
            HAVING count(*) > 1
            "#,
        )?;
        let tmdb_ids: Vec<i64> = tmdb_stmt
            .query_map([], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        for id in tmdb_ids {
            let mut item_stmt = conn.prepare(
                r#"
                SELECT m.id, m.library_root_id, m.folder_path, m.file_path, m.title, m.year,
                       m.resolution, m.imdb_id, m.tmdb_id, m.file_size, m.version_label,
                       ms.video_codec, ms.audio_codec, ms.bitrate_kbps
                FROM movies m
                LEFT JOIN media_streams ms ON ms.media_type = 'movie' AND ms.media_id = m.id
                WHERE m.tmdb_id = ?1
                ORDER BY m.file_size DESC
                "#,
            )?;
            let items: Vec<DuplicateMovieItem> = item_stmt
                .query_map(params![id], |row| {
                    Ok(DuplicateMovieItem {
                        id: row.get(0)?,
                        library_root_id: row.get(1)?,
                        folder_path: row.get(2)?,
                        file_path: row.get(3)?,
                        title: row.get(4)?,
                        year: row.get(5)?,
                        resolution: row.get(6)?,
                        imdb_id: row.get(7)?,
                        tmdb_id: row.get(8)?,
                        file_size: row.get(9)?,
                        version_label: row.get(10)?,
                        video_codec: row.get(11)?,
                        audio_codec: row.get(12)?,
                        bitrate_kbps: row.get(13)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            if items.len() > 1 {
                groups.push(MovieDuplicateGroup {
                    match_key: format!("tmdb:{}", id),
                    reason: format!("Identical TMDb ID ({})", id),
                    items,
                });
            }
        }
    }

    // 3. Group by Normalized Title + Year (for unindexed movies)
    {
        let mut title_stmt = conn.prepare(
            r#"
            SELECT lower(trim(title)) as norm_title, year
            FROM movies
            WHERE (imdb_id IS NULL OR imdb_id = '') AND tmdb_id IS NULL AND year IS NOT NULL
            GROUP BY norm_title, year
            HAVING count(*) > 1
            "#,
        )?;
        let title_pairs: Vec<(String, i32)> = title_stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        for (norm_title, year) in title_pairs {
            let mut item_stmt = conn.prepare(
                r#"
                SELECT m.id, m.library_root_id, m.folder_path, m.file_path, m.title, m.year,
                       m.resolution, m.imdb_id, m.tmdb_id, m.file_size, m.version_label,
                       ms.video_codec, ms.audio_codec, ms.bitrate_kbps
                FROM movies m
                LEFT JOIN media_streams ms ON ms.media_type = 'movie' AND ms.media_id = m.id
                WHERE lower(trim(m.title)) = ?1 AND m.year = ?2
                ORDER BY m.file_size DESC
                "#,
            )?;
            let items: Vec<DuplicateMovieItem> = item_stmt
                .query_map(params![norm_title, year], |row| {
                    Ok(DuplicateMovieItem {
                        id: row.get(0)?,
                        library_root_id: row.get(1)?,
                        folder_path: row.get(2)?,
                        file_path: row.get(3)?,
                        title: row.get(4)?,
                        year: row.get(5)?,
                        resolution: row.get(6)?,
                        imdb_id: row.get(7)?,
                        tmdb_id: row.get(8)?,
                        file_size: row.get(9)?,
                        version_label: row.get(10)?,
                        video_codec: row.get(11)?,
                        audio_codec: row.get(12)?,
                        bitrate_kbps: row.get(13)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            if items.len() > 1 {
                let first_title = items[0].title.clone();
                groups.push(MovieDuplicateGroup {
                    match_key: format!("title:{}:{}", norm_title, year),
                    reason: format!("Identical Title & Year ({} - {})", first_title, year),
                    items,
                });
            }
        }
    }

    Ok(groups)
}

pub fn find_episode_duplicates(conn: &Connection) -> Result<Vec<EpisodeDuplicateGroup>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT e.tv_show_id, s.title, e.season, e.episode
        FROM episodes e
        JOIN tv_shows s ON s.id = e.tv_show_id
        GROUP BY e.tv_show_id, e.season, e.episode
        HAVING count(*) > 1
        "#,
    )?;

    let groups_meta: Vec<(i64, String, i32, i32)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut result = Vec::new();

    for (tv_show_id, show_title, season, episode) in groups_meta {
        let mut item_stmt = conn.prepare(
            r#"
            SELECT e.id, e.tv_show_id, ?1, e.file_path, e.season, e.episode, e.title,
                   e.resolution, e.file_size, ms.video_codec, ms.audio_codec, ms.bitrate_kbps
            FROM episodes e
            LEFT JOIN media_streams ms ON ms.media_type = 'episode' AND ms.media_id = e.id
            WHERE e.tv_show_id = ?2 AND e.season = ?3 AND e.episode = ?4
            ORDER BY e.file_size DESC
            "#,
        )?;

        let items: Vec<DuplicateEpisodeItem> = item_stmt
            .query_map(params![show_title, tv_show_id, season, episode], |row| {
                Ok(DuplicateEpisodeItem {
                    id: row.get(0)?,
                    tv_show_id: row.get(1)?,
                    show_title: row.get(2)?,
                    file_path: row.get(3)?,
                    season: row.get(4)?,
                    episode: row.get(5)?,
                    title: row.get(6)?,
                    resolution: row.get(7)?,
                    file_size: row.get(8)?,
                    video_codec: row.get(9)?,
                    audio_codec: row.get(10)?,
                    bitrate_kbps: row.get(11)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        if items.len() > 1 {
            result.push(EpisodeDuplicateGroup {
                tv_show_id,
                show_title: show_title.clone(),
                season,
                episode,
                match_key: format!("{}:S{:02}E{:02}", tv_show_id, season, episode),
                items,
            });
        }
    }

    Ok(result)
}
