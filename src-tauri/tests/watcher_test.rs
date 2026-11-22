use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use peppy_media_organizer::db::{self, queries};
use peppy_media_organizer::scanner::watcher::{is_relevant_path, LibraryWatcher};

#[test]
fn test_watcher_filtering_rules() {
    // Valid video files
    assert!(is_relevant_path(Path::new("/movies/Movie (2020)/Movie (2020).mkv")));
    assert!(is_relevant_path(Path::new("/movies/Movie (2020)/Movie (2020).mp4")));

    // Sidecar files
    assert!(is_relevant_path(Path::new("/movies/Movie (2020)/Movie (2020).en.srt")));
    assert!(is_relevant_path(Path::new("/movies/Movie (2020)/movie.nfo")));
    assert!(is_relevant_path(Path::new("/movies/Movie (2020)/poster.jpg")));
    assert!(is_relevant_path(Path::new("/movies/Movie (2020)/backdrop.png")));
    assert!(is_relevant_path(Path::new("/movies/Movie (2020)/season01-poster.jpg")));

    // Directories
    assert!(is_relevant_path(Path::new("/movies/New Movie (2024)")));

    // Ignored / transient
    assert!(!is_relevant_path(Path::new("/movies/.git/config")));
    assert!(!is_relevant_path(Path::new("/movies/.DS_Store")));
    assert!(!is_relevant_path(Path::new("/movies/Movie (2020)/movie.mkv.tmp")));
    assert!(!is_relevant_path(Path::new("/movies/Movie (2020)/movie.mkv.crdownload")));
    assert!(!is_relevant_path(Path::new("/movies/Movie (2020)/movie.mkv.part")));
    assert!(!is_relevant_path(Path::new("/movies/Movie (2020)/movie.sample.mkv")));
    assert!(!is_relevant_path(Path::new("/movies/Movie (2020)/trailers/trailer.mkv")));
    assert!(!is_relevant_path(Path::new("/movies/Movie (2020)/extras/extra.mkv")));
    assert!(!is_relevant_path(Path::new("/movies/Movie (2020)/deleted scenes/scene1.mkv")));
}

#[test]
fn test_watcher_lifecycle_and_status() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_watcher_test_{}", uuid::Uuid::new_v4()));
    let movie_dir = temp_dir.join("Movies");
    fs::create_dir_all(&movie_dir).unwrap();

    let conn = db::init_db(&temp_dir).expect("Failed to initialize DB");
    let db_arc = Arc::new(Mutex::new(conn));

    let root_id = {
        let conn_guard = db_arc.lock().unwrap();
        queries::insert_library_root(&conn_guard, movie_dir.to_str().unwrap(), "movie").unwrap()
    };
    assert!(root_id > 0);

    let roots = {
        let conn_guard = db_arc.lock().unwrap();
        queries::get_library_roots(&conn_guard).unwrap()
    };
    assert_eq!(roots.len(), 1);

    let mut watcher = LibraryWatcher::new();
    let status_initial = watcher.get_status();
    assert!(status_initial.enabled);
    assert_eq!(status_initial.watched_roots_count, 0);

    // Start watching
    watcher.start(roots.clone(), None, db_arc.clone()).expect("Failed to start watcher");
    let status_running = watcher.get_status();
    assert!(status_running.enabled);
    assert_eq!(status_running.watched_roots_count, 1);
    assert_eq!(status_running.watched_paths, vec![movie_dir.to_str().unwrap().to_string()]);

    // Disable watcher
    watcher.set_enabled(false, roots.clone(), None, db_arc.clone()).unwrap();
    let status_disabled = watcher.get_status();
    assert!(!status_disabled.enabled);
    assert_eq!(status_disabled.watched_roots_count, 0);

    // Re-enable watcher
    watcher.set_enabled(true, roots, None, db_arc.clone()).unwrap();
    let status_re_enabled = watcher.get_status();
    assert!(status_re_enabled.enabled);
    assert_eq!(status_re_enabled.watched_roots_count, 1);

    // Stop watcher
    watcher.stop();
    let _ = fs::remove_dir_all(&temp_dir);
}
