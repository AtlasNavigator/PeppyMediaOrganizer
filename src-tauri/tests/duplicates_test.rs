use peppy_media_organizer::db::{self, models::*, queries};

#[test]
fn test_movie_duplicate_detection() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_dup_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let conn = db::init_db(&temp_dir).unwrap();
    let root_id = queries::insert_library_root(&conn, "/media/movies", "movie").unwrap();

    // 1. Insert multi-version movies with same IMDb ID
    let br1 = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: "/media/movies/Blade Runner (1982)".to_string(),
        file_path: "/media/movies/Blade Runner (1982)/Blade Runner 1080p.mkv".to_string(),
        title: "Blade Runner".to_string(),
        sort_title: None,
        year: Some(1982),
        resolution: Some("1080p".to_string()),
        imdb_id: Some("tt0083658".to_string()),
        tmdb_id: None,
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 1,
        file_size: Some(8_000_000_000),
        subtitle_count: 1,
        version_label: Some("1080p Theatrical".to_string()),
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_movie(&conn, &br1).unwrap();

    let br2 = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: "/media/movies/Blade Runner (1982)".to_string(),
        file_path: "/media/movies/Blade Runner (1982)/Blade Runner 2160p.mkv".to_string(),
        title: "Blade Runner".to_string(),
        sort_title: None,
        year: Some(1982),
        resolution: Some("2160p".to_string()),
        imdb_id: Some("tt0083658".to_string()),
        tmdb_id: None,
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 1,
        file_size: Some(25_000_000_000),
        subtitle_count: 1,
        version_label: Some("2160p Final Cut".to_string()),
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_movie(&conn, &br2).unwrap();

    // 2. Insert duplicate without IMDb ID but same title & year
    let alien1 = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: "/media/movies/Alien (1979)".to_string(),
        file_path: "/media/movies/Alien (1979)/alien.mkv".to_string(),
        title: "Alien".to_string(),
        sort_title: None,
        year: Some(1979),
        resolution: Some("1080p".to_string()),
        imdb_id: None,
        tmdb_id: None,
        has_nfo: 0,
        has_poster: 0,
        has_backdrop: 0,
        file_size: Some(6_000_000_000),
        subtitle_count: 0,
        version_label: None,
        scan_status: "unmatched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_movie(&conn, &alien1).unwrap();

    let alien2 = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: "/media/movies/Alien 1979".to_string(),
        file_path: "/media/movies/Alien 1979/alien-copy.mkv".to_string(),
        title: "Alien".to_string(),
        sort_title: None,
        year: Some(1979),
        resolution: Some("720p".to_string()),
        imdb_id: None,
        tmdb_id: None,
        has_nfo: 0,
        has_poster: 0,
        has_backdrop: 0,
        file_size: Some(3_000_000_000),
        subtitle_count: 0,
        version_label: None,
        scan_status: "unmatched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_movie(&conn, &alien2).unwrap();

    // 3. Insert non-duplicate movie
    let matrix = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: "/media/movies/The Matrix (1999)".to_string(),
        file_path: "/media/movies/The Matrix (1999)/matrix.mkv".to_string(),
        title: "The Matrix".to_string(),
        sort_title: None,
        year: Some(1999),
        resolution: Some("1080p".to_string()),
        imdb_id: Some("tt0133093".to_string()),
        tmdb_id: None,
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 1,
        file_size: Some(10_000_000_000),
        subtitle_count: 1,
        version_label: None,
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_movie(&conn, &matrix).unwrap();

    // 4. Query duplicates
    let dups = queries::find_movie_duplicates(&conn).unwrap();
    assert_eq!(dups.len(), 2, "Expected exactly 2 duplicate groups");

    let br_group = dups.iter().find(|g| g.match_key == "imdb:tt0083658").expect("Blade Runner group found");
    assert_eq!(br_group.items.len(), 2);
    assert_eq!(br_group.items[0].file_size, Some(25_000_000_000)); // Sorted by file_size DESC

    let alien_group = dups.iter().find(|g| g.match_key.starts_with("title:alien:1979")).expect("Alien group found");
    assert_eq!(alien_group.items.len(), 2);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_episode_duplicate_detection() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_ep_dup_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let conn = db::init_db(&temp_dir).unwrap();
    let root_id = queries::insert_library_root(&conn, "/media/tv", "tvshow").unwrap();

    let show = TvShow {
        id: 0,
        library_root_id: root_id,
        folder_path: "/media/tv/Breaking Bad".to_string(),
        title: "Breaking Bad".to_string(),
        sort_title: None,
        year: Some(2008),
        tvdb_id: Some(81189),
        tmdb_id: None,
        imdb_id: None,
        status: None,
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 1,
        season_count: 1,
        episode_count: 2,
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    let show_id = queries::insert_tv_show(&conn, &show).unwrap();

    // Episode 1 1080p
    let ep1 = Episode {
        id: 0,
        tv_show_id: show_id,
        file_path: "/media/tv/Breaking Bad/Season 01/BB S01E01 1080p.mkv".to_string(),
        title: Some("Pilot".to_string()),
        season: 1,
        episode: 1,
        episode_end: None,
        resolution: Some("1080p".to_string()),
        imdb_id: None,
        tvdb_id: None,
        has_nfo: 1,
        file_size: Some(2_000_000_000),
        subtitle_count: 1,
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_episode(&conn, &ep1).unwrap();

    // Episode 1 720p duplicate
    let ep2 = Episode {
        id: 0,
        tv_show_id: show_id,
        file_path: "/media/tv/Breaking Bad/Season 01/BB S01E01 720p.mkv".to_string(),
        title: Some("Pilot".to_string()),
        season: 1,
        episode: 1,
        episode_end: None,
        resolution: Some("720p".to_string()),
        imdb_id: None,
        tvdb_id: None,
        has_nfo: 0,
        file_size: Some(900_000_000),
        subtitle_count: 0,
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_episode(&conn, &ep2).unwrap();

    let dups = queries::find_episode_duplicates(&conn).unwrap();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].season, 1);
    assert_eq!(dups[0].episode, 1);
    assert_eq!(dups[0].items.len(), 2);
    assert_eq!(dups[0].items[0].resolution.as_deref(), Some("1080p"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_episodes_migration_allows_duplicate_episodes() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_ep_mig_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let db_path = temp_dir.join("library.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();

    // 1. Manually create legacy schema that enforced composite UNIQUE(tv_show_id, season, episode)
    conn.execute_batch(r#"
        CREATE TABLE library_roots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            media_type TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            last_scanned_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE tv_shows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            library_root_id INTEGER NOT NULL REFERENCES library_roots(id),
            folder_path TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            sort_title TEXT,
            year INTEGER,
            tvdb_id INTEGER,
            tmdb_id INTEGER,
            imdb_id TEXT,
            status TEXT,
            has_nfo INTEGER NOT NULL DEFAULT 0,
            has_poster INTEGER NOT NULL DEFAULT 0,
            has_backdrop INTEGER NOT NULL DEFAULT 0,
            season_count INTEGER NOT NULL DEFAULT 0,
            episode_count INTEGER NOT NULL DEFAULT 0,
            scan_status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE episodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tv_show_id INTEGER NOT NULL REFERENCES tv_shows(id) ON DELETE CASCADE,
            file_path TEXT NOT NULL UNIQUE,
            title TEXT,
            season INTEGER NOT NULL,
            episode INTEGER NOT NULL,
            episode_end INTEGER,
            resolution TEXT,
            imdb_id TEXT,
            tvdb_id INTEGER,
            has_nfo INTEGER NOT NULL DEFAULT 0,
            file_size INTEGER,
            subtitle_count INTEGER NOT NULL DEFAULT 0,
            scan_status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(tv_show_id, season, episode)
        );
    "#).unwrap();

    // 2. Run migrations — this should detect and rebuild episodes without composite unique
    db::migrations::run_migrations(&conn).unwrap();

    let root_id = queries::insert_library_root(&conn, "/media/tv", "tvshow").unwrap();
    let show = TvShow {
        id: 0,
        library_root_id: root_id,
        folder_path: "/media/tv/The Wire".to_string(),
        title: "The Wire".to_string(),
        sort_title: None,
        year: Some(2002),
        tvdb_id: Some(79125),
        tmdb_id: None,
        imdb_id: None,
        status: None,
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 1,
        season_count: 1,
        episode_count: 2,
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    let show_id = queries::insert_tv_show(&conn, &show).unwrap();

    // 3. Insert two different cuts of the same episode S01E01
    let ep1 = Episode {
        id: 0,
        tv_show_id: show_id,
        file_path: "/media/tv/The Wire/Season 01/The Wire S01E01 1080p Remaster.mkv".to_string(),
        title: Some("The Target".to_string()),
        season: 1,
        episode: 1,
        episode_end: None,
        resolution: Some("1080p".to_string()),
        imdb_id: None,
        tvdb_id: None,
        has_nfo: 1,
        file_size: Some(3_500_000_000),
        subtitle_count: 1,
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    let ep1_id = queries::insert_episode(&conn, &ep1).expect("Ep 1 insertion should succeed");
    assert!(ep1_id > 0);

    let ep2 = Episode {
        id: 0,
        tv_show_id: show_id,
        file_path: "/media/tv/The Wire/Season 01/The Wire S01E01 480p Broadcast.mkv".to_string(),
        title: Some("The Target".to_string()),
        season: 1,
        episode: 1,
        episode_end: None,
        resolution: Some("480p".to_string()),
        imdb_id: None,
        tvdb_id: None,
        has_nfo: 0,
        file_size: Some(700_000_000),
        subtitle_count: 0,
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    let ep2_id = queries::insert_episode(&conn, &ep2).expect("Duplicate Ep 2 insertion must succeed after migration");
    assert!(ep2_id > 0);

    // 4. Verify find_episode_duplicates returns this group with both candidates
    let dups = queries::find_episode_duplicates(&conn).unwrap();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].show_title, "The Wire");
    assert_eq!(dups[0].season, 1);
    assert_eq!(dups[0].episode, 1);
    assert_eq!(dups[0].items.len(), 2);
    assert_eq!(dups[0].items[0].resolution.as_deref(), Some("1080p"));
    assert_eq!(dups[0].items[1].resolution.as_deref(), Some("480p"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_plan_resolve_duplicates_and_reversible_undo() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_dup_resolve_test_{}", uuid::Uuid::new_v4()));
    let movie_dir = temp_dir.join("Movies").join("Alien (1979)");
    std::fs::create_dir_all(&movie_dir).unwrap();

    let alien_keep = movie_dir.join("Alien (1979) [imdbid-tt0078748] - 2160p.mkv");
    let alien_dup = movie_dir.join("Alien (1979) - 720p.mkv");
    let alien_dup_srt = movie_dir.join("Alien (1979) - 720p.en.srt");

    std::fs::write(&alien_keep, b"primary 4k file content").unwrap();
    std::fs::write(&alien_dup, b"duplicate 720p file content").unwrap();
    std::fs::write(&alien_dup_srt, b"subtitles content").unwrap();

    let mut conn = db::init_db(&temp_dir).unwrap();
    let root_id = queries::insert_library_root(&conn, temp_dir.join("Movies").to_str().unwrap(), "movie").unwrap();

    let m1 = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: movie_dir.to_str().unwrap().to_string(),
        file_path: alien_keep.to_str().unwrap().to_string(),
        title: "Alien".to_string(),
        sort_title: None,
        year: Some(1979),
        resolution: Some("2160p".to_string()),
        imdb_id: Some("tt0078748".to_string()),
        tmdb_id: None,
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 1,
        file_size: Some(25_000_000),
        subtitle_count: 0,
        version_label: Some("2160p".to_string()),
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_movie(&conn, &m1).unwrap();

    let m2 = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: movie_dir.to_str().unwrap().to_string(),
        file_path: alien_dup.to_str().unwrap().to_string(),
        title: "Alien".to_string(),
        sort_title: None,
        year: Some(1979),
        resolution: Some("720p".to_string()),
        imdb_id: Some("tt0078748".to_string()),
        tmdb_id: None,
        has_nfo: 0,
        has_poster: 0,
        has_backdrop: 0,
        file_size: Some(5_000_000),
        subtitle_count: 1,
        version_label: Some("720p".to_string()),
        scan_status: "matched".to_string(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    queries::insert_movie(&conn, &m2).unwrap();

    // 1. Generate resolve duplicates plan
    let plan = peppy_media_organizer::renamer::plan_resolve_duplicates(
        &conn,
        alien_keep.to_str().unwrap(),
        &[alien_dup.to_str().unwrap().to_string()],
        "movie",
    ).unwrap();

    assert_eq!(plan.will_rename_count, 2); // 1 video + 1 sidecar
    assert_eq!(plan.conflict_count, 0);

    // Verify destination points to _duplicates_archive subfolder
    let archive_video_action = plan.actions.iter().find(|a| a.old_path == alien_dup.to_str().unwrap()).unwrap();
    assert!(archive_video_action.new_path.contains("_duplicates_archive"));

    let archive_srt_action = plan.actions.iter().find(|a| a.old_path == alien_dup_srt.to_str().unwrap()).unwrap();
    assert!(archive_srt_action.new_path.contains("_duplicates_archive"));

    // 2. Execute plan
    let exec_res = peppy_media_organizer::renamer::execute_rename_plan(&mut conn, &plan, None).unwrap();
    assert_eq!(exec_res.executed_count, 2);
    assert_eq!(exec_res.failed_count, 0);

    // Assert files moved to _duplicates_archive
    assert!(!alien_dup.exists(), "Original duplicate video should be moved");
    assert!(!alien_dup_srt.exists(), "Original duplicate srt should be moved");

    let archived_dup_path = movie_dir.join("_duplicates_archive").join("Alien (1979) - 720p.mkv");
    let archived_srt_path = movie_dir.join("_duplicates_archive").join("Alien (1979) - 720p.en.srt");
    assert!(archived_dup_path.exists(), "Archived duplicate video must exist");
    assert!(archived_srt_path.exists(), "Archived duplicate srt must exist");
    assert!(alien_keep.exists(), "Kept primary video must remain untouched");

    // 3. Undo Last Rename
    let undo_res = peppy_media_organizer::renamer::undo_last_rename(&mut conn).unwrap();
    assert_eq!(undo_res.reverted_count, 2);
    assert_eq!(undo_res.failed_count, 0);

    // Assert files restored back
    assert!(alien_dup.exists(), "Duplicate video must be restored back to original location");
    assert!(alien_dup_srt.exists(), "Duplicate srt must be restored back to original location");

    let _ = std::fs::remove_dir_all(&temp_dir);
}
