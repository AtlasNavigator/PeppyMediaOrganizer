use std::fs::{self, File};
use std::path::Path;
use uuid::Uuid;

use peppy_media_organizer::db::init_db;
use peppy_media_organizer::db::models::{Episode, Movie, TvShow};
use peppy_media_organizer::db::queries::{insert_episode, insert_library_root, insert_movie, insert_tv_show};
use peppy_media_organizer::naming::jellyfin::{
    format_episode_filename, format_movie_filename, format_season_folder,
};
use peppy_media_organizer::naming::sanitize::sanitize_path_component;
use peppy_media_organizer::renamer::{execute_rename_plan, plan_movie_rename, plan_tv_show_rename, undo_last_rename};

fn touch(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    File::create(path).unwrap();
}

#[test]
fn test_illegal_character_sanitization() {
    // Colon replaced with " -"
    assert_eq!(sanitize_path_component("Dune: Part Two"), "Dune - Part Two");
    assert_eq!(sanitize_path_component("Mission: Impossible: Rogue Nation"), "Mission - Impossible - Rogue Nation");

    // Slashes replaced with "-"
    assert_eq!(sanitize_path_component("Face/Off"), "Face-Off");
    assert_eq!(sanitize_path_component(r"Batman\Superman"), "Batman-Superman");

    // Illegal OS chars stripped: < > " | ? *
    assert_eq!(
        sanitize_path_component("Who Framed Roger Rabbit? <1988> *\"special\"|"),
        "Who Framed Roger Rabbit 1988 special"
    );

    // Trailing dots and spaces stripped (Windows NTFS forbidden)
    assert_eq!(sanitize_path_component("Se7en...   "), "Se7en");
    assert_eq!(sanitize_path_component("Wall-E . ."), "Wall-E");
}

#[test]
fn test_multi_version_movie_naming() {
    let folder_name = "Blade Runner (1982) [imdbid-tt0083658]";

    // Single version has no suffix
    assert_eq!(
        format_movie_filename(folder_name, None, "mkv"),
        "Blade Runner (1982) [imdbid-tt0083658].mkv"
    );

    // Multi-version movie appends " - Label" where prefix exactly matches parent folder name
    assert_eq!(
        format_movie_filename(folder_name, Some("1080p Theatrical"), "mkv"),
        "Blade Runner (1982) [imdbid-tt0083658] - 1080p Theatrical.mkv"
    );
    assert_eq!(
        format_movie_filename(folder_name, Some("2160p Final Cut"), "mkv"),
        "Blade Runner (1982) [imdbid-tt0083658] - 2160p Final Cut.mkv"
    );
}

#[test]
fn test_tv_specials_and_multi_episode_naming() {
    // Season 00 for specials
    assert_eq!(format_season_folder(0), "Season 00");
    assert_eq!(format_season_folder(1), "Season 01");
    assert_eq!(format_season_folder(12), "Season 12");

    // Special episode S00E01
    assert_eq!(
        format_episode_filename("The Mandalorian", 0, 1, None, Some("Disney Gallery"), "mkv"),
        "The Mandalorian S00E01 - Disney Gallery.mkv"
    );

    // Multi-part episode S05E15-E16
    assert_eq!(
        format_episode_filename(
            "Breaking Bad",
            5,
            15,
            Some(16),
            Some("Granite State & Felina"),
            "mkv"
        ),
        "Breaking Bad S05E15-E16 - Granite State & Felina.mkv"
    );

    // Shows with >99 episodes use 3-digit zero-padding
    assert_eq!(
        format_episode_filename("One Piece", 1, 105, None, Some("Grand Line"), "mkv"),
        "One Piece S01E105 - Grand Line.mkv"
    );
    assert_eq!(
        format_episode_filename("One Piece", 1, 105, Some(106), Some("Battle"), "mkv"),
        "One Piece S01E105-E106 - Battle.mkv"
    );
}

#[test]
fn test_round_trip_movie_rename_and_undo() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_renamer_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let _db_path = temp_dir.join("test_app.db");
    let mut conn = init_db(&temp_dir).expect("Failed to init DB");

    let movies_root = temp_dir.join("Movies");
    fs::create_dir_all(&movies_root).unwrap();

    let root_id = insert_library_root(&conn, &movies_root.to_string_lossy(), "movie").unwrap();

    // Create a messy movie folder on disk with sidecars
    let old_movie_folder = movies_root.join("the.matrix.1999.1080p.bluray.x264");
    let old_video_file = old_movie_folder.join("the.matrix.1999.1080p.bluray.x264.mkv");
    let old_sub_file = old_movie_folder.join("the.matrix.1999.1080p.bluray.x264.en.forced.srt");
    let old_nfo_file = old_movie_folder.join("movie.nfo");
    let old_poster_file = old_movie_folder.join("poster.jpg");

    touch(&old_video_file);
    touch(&old_sub_file);
    touch(&old_nfo_file);
    touch(&old_poster_file);

    // Insert into DB
    let movie = Movie {
        id: 0,
        library_root_id: root_id,
        folder_path: old_movie_folder.to_string_lossy().to_string(),
        file_path: old_video_file.to_string_lossy().to_string(),
        title: "The Matrix".into(),
        sort_title: None,
        year: Some(1999),
        resolution: Some("1080p".into()),
        imdb_id: Some("tt0133093".into()),
        tmdb_id: Some(603),
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 0,
        file_size: Some(1024),
        subtitle_count: 1,
        version_label: None,
        scan_status: "matched".into(),
        created_at: "now".into(),
        updated_at: "now".into(),
    };
    let movie_id = insert_movie(&conn, &movie).unwrap();

    // 1. Generate Rename Plan
    let plan = plan_movie_rename(&conn, movie_id).expect("Failed to plan rename");
    assert!(plan.will_rename_count >= 4, "Should plan to rename folder, video, sub, and nfo/poster");

    let expected_folder = movies_root.join("The Matrix (1999) [imdbid-tt0133093]");
    let expected_video = expected_folder.join("The Matrix (1999) [imdbid-tt0133093].mkv");
    let expected_sub = expected_folder.join("The Matrix (1999) [imdbid-tt0133093].en.forced.srt");
    let expected_nfo = expected_folder.join("movie.nfo");
    let expected_poster = expected_folder.join("poster.jpg");

    // 2. Execute Rename Plan
    let exec_res = execute_rename_plan(&mut conn, &plan, None).expect("Failed to execute rename");
    assert_eq!(exec_res.failed_count, 0, "No actions should fail");
    assert!(exec_res.executed_count >= 4);

    // Verify files at new paths
    assert!(expected_folder.exists(), "Target folder should exist");
    assert!(expected_video.exists(), "Target video file should exist");
    assert!(expected_sub.exists(), "Target subtitle file should exist");
    assert!(expected_nfo.exists(), "Target movie.nfo should exist");
    assert!(expected_poster.exists(), "Target poster.jpg should exist");

    // Verify old files no longer exist
    assert!(!old_video_file.exists(), "Old video should not exist");
    assert!(!old_movie_folder.exists(), "Old folder should be removed");

    // Verify DB updated
    let movie_db: (String, String) = conn
        .query_row(
            "SELECT folder_path, file_path FROM movies WHERE id = ?1",
            rusqlite::params![movie_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(movie_db.0, expected_folder.to_string_lossy());
    assert_eq!(movie_db.1, expected_video.to_string_lossy());

    // 3. Undo Last Rename
    let undo_res = undo_last_rename(&mut conn).expect("Failed to undo rename");
    assert_eq!(undo_res.failed_count, 0, "Undo should have 0 failures");
    assert!(undo_res.reverted_count >= 4);

    // Verify original paths restored
    assert!(old_movie_folder.exists(), "Original movie folder should be restored");
    assert!(old_video_file.exists(), "Original video should be restored");
    assert!(old_sub_file.exists(), "Original subtitle should be restored");
    assert!(old_nfo_file.exists(), "Original movie.nfo should be restored");
    assert!(old_poster_file.exists(), "Original poster.jpg should be restored");

    // Verify target paths no longer exist
    assert!(!expected_video.exists(), "Target video should no longer exist");
    assert!(!expected_folder.exists(), "Target folder should no longer exist");

    // Verify DB reverted
    let reverted_db: (String, String) = conn
        .query_row(
            "SELECT folder_path, file_path FROM movies WHERE id = ?1",
            rusqlite::params![movie_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(reverted_db.0, old_movie_folder.to_string_lossy());
    assert_eq!(reverted_db.1, old_video_file.to_string_lossy());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_round_trip_tv_show_rename_and_undo() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_tv_renamer_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let _db_path = temp_dir.join("test_tv_app.db");
    let mut conn = init_db(&temp_dir).expect("Failed to init DB");

    let tv_root = temp_dir.join("TV Shows");
    fs::create_dir_all(&tv_root).unwrap();

    let root_id = insert_library_root(&conn, &tv_root.to_string_lossy(), "tvshow").unwrap();

    // Create a messy TV show folder: better.call.saul
    let old_show_folder = tv_root.join("better.call.saul");
    let old_tv_nfo = old_show_folder.join("tvshow.nfo");
    let old_poster = old_show_folder.join("poster.jpg");

    let old_season_folder = old_show_folder.join("season 1");
    let old_ep_file = old_season_folder.join("better.call.saul.s01e01.1080p.mkv");
    let old_ep_sub = old_season_folder.join("better.call.saul.s01e01.1080p.en.srt");
    let old_ep_nfo = old_season_folder.join("better.call.saul.s01e01.1080p.nfo");

    touch(&old_tv_nfo);
    touch(&old_poster);
    touch(&old_ep_file);
    touch(&old_ep_sub);
    touch(&old_ep_nfo);

    let show = TvShow {
        id: 0,
        library_root_id: root_id,
        folder_path: old_show_folder.to_string_lossy().to_string(),
        title: "Better Call Saul".into(),
        sort_title: None,
        year: Some(2015),
        tvdb_id: Some(273181),
        tmdb_id: Some(60059),
        imdb_id: Some("tt3032476".into()),
        status: Some("Ended".into()),
        has_nfo: 1,
        has_poster: 1,
        has_backdrop: 0,
        season_count: 1,
        episode_count: 1,
        scan_status: "matched".into(),
        created_at: "now".into(),
        updated_at: "now".into(),
    };
    let show_id = insert_tv_show(&conn, &show).unwrap();

    let ep = Episode {
        id: 0,
        tv_show_id: show_id,
        file_path: old_ep_file.to_string_lossy().to_string(),
        title: Some("Uno".into()),
        season: 1,
        episode: 1,
        episode_end: None,
        resolution: Some("1080p".into()),
        imdb_id: Some("tt3464768".into()),
        tvdb_id: Some(4931980),
        has_nfo: 1,
        file_size: Some(1024),
        subtitle_count: 1,
        scan_status: "matched".into(),
        created_at: "now".into(),
        updated_at: "now".into(),
    };
    let _ep_id = insert_episode(&conn, &ep).unwrap();

    // 1. Plan TV Show Rename
    let plan = plan_tv_show_rename(&conn, show_id).expect("Failed to plan TV rename");
    assert!(plan.will_rename_count >= 5);

    let expected_show_folder = tv_root.join("Better Call Saul (2015) [tvdbid-273181]");
    let expected_season_folder = expected_show_folder.join("Season 01");
    let expected_ep_file = expected_season_folder.join("Better Call Saul S01E01 - Uno.mkv");
    let expected_ep_sub = expected_season_folder.join("Better Call Saul S01E01 - Uno.en.srt");
    let expected_ep_nfo = expected_season_folder.join("Better Call Saul S01E01 - Uno.nfo");

    // 2. Execute Rename Plan
    let exec_res = execute_rename_plan(&mut conn, &plan, None).expect("Failed to execute TV rename");
    assert_eq!(exec_res.failed_count, 0);

    assert!(expected_show_folder.exists());
    assert!(expected_season_folder.exists());
    assert!(expected_ep_file.exists());
    assert!(expected_ep_sub.exists());
    assert!(expected_ep_nfo.exists());

    // 3. Undo Last Rename
    let undo_res = undo_last_rename(&mut conn).expect("Failed to undo TV rename");
    assert_eq!(undo_res.failed_count, 0);

    assert!(old_show_folder.exists());
    assert!(old_ep_file.exists());
    assert!(old_ep_sub.exists());
    assert!(old_ep_nfo.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}
