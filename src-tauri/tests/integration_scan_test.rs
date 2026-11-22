use std::sync::Mutex;
use peppy_media_organizer::db::{self, queries};
use peppy_media_organizer::scanner::walker::{scan_movie_root, scan_tvshow_root};

#[test]
fn test_end_to_end_library_scan() {
    let temp_dir = std::env::temp_dir().join(format!("peppy_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Initialize SQLite database
    let conn = db::init_db(&temp_dir).expect("Failed to initialize database");
    let conn_mutex = Mutex::new(conn);

    // 2. Identify test-library paths
    let workspace_root = std::env::current_dir().unwrap().parent().unwrap().to_path_buf();
    let movies_path = workspace_root.join("test-library").join("Movies");
    let tvshows_path = workspace_root.join("test-library").join("TV Shows");

    assert!(movies_path.exists(), "test-library/Movies must exist");
    assert!(tvshows_path.exists(), "test-library/TV Shows must exist");

    // 3. Insert library roots
    let movie_root_id = {
        let conn = conn_mutex.lock().unwrap();
        queries::insert_library_root(&conn, movies_path.to_str().unwrap(), "movie")
            .expect("Failed to insert movie root")
    };

    let tv_root_id = {
        let conn = conn_mutex.lock().unwrap();
        queries::insert_library_root(&conn, tvshows_path.to_str().unwrap(), "tvshow")
            .expect("Failed to insert tvshow root")
    };

    // 4. Execute Scanner on Movies root
    let movie_scan_result = scan_movie_root(
        movies_path.to_str().unwrap(),
        movie_root_id,
        &conn_mutex,
        None,
    );

    println!(
        "Movie scan complete: {} movies found in {}ms",
        movie_scan_result.movies_found, movie_scan_result.duration_ms
    );
    assert!(movie_scan_result.movies_found >= 50, "Should find at least 50 movies");

    // 5. Execute Scanner on TV Shows root
    let tv_scan_result = scan_tvshow_root(
        tvshows_path.to_str().unwrap(),
        tv_root_id,
        &conn_mutex,
        None,
    );

    println!(
        "TV scan complete: {} shows, {} episodes found in {}ms",
        tv_scan_result.shows_found, tv_scan_result.episodes_found, tv_scan_result.duration_ms
    );
    assert_eq!(tv_scan_result.shows_found, 10, "Should find 10 TV shows");
    assert!(tv_scan_result.episodes_found >= 100, "Should find >100 episodes");

    // 6. Verify SQLite Database State
    {
        let conn = conn_mutex.lock().unwrap();

        // Check movies
        let movies = queries::get_movies(&conn, Some(movie_root_id)).unwrap();
        assert_eq!(movies.len(), movie_scan_result.movies_found);

        // Verify Alien metadata
        let alien = movies.iter().find(|m| m.title == "Alien").expect("Alien should be in DB");
        assert_eq!(alien.year, Some(1979));
        assert_eq!(alien.imdb_id.as_deref(), Some("tt0078748"));
        assert_eq!(alien.has_nfo, 1);
        assert_eq!(alien.has_poster, 1);
        assert_eq!(alien.scan_status, "matched");

        // Verify Multi-Version Blade Runner
        let br = movies.iter().find(|m| m.title == "Blade Runner").expect("Blade Runner should be in DB");
        assert_eq!(br.year, Some(1982));
        assert_eq!(br.imdb_id.as_deref(), Some("tt0083658"));

        // Verify Messy Scene Movie (e.g. the shining)
        let shining = movies.iter().find(|m| m.title.to_lowercase().contains("shining")).expect("Shining should be in DB");
        assert_eq!(shining.year, Some(1980));
        assert_eq!(shining.resolution.as_deref(), Some("1080p"));

        // Check TV Shows
        let shows = queries::get_tv_shows(&conn, Some(tv_root_id)).unwrap();
        assert_eq!(shows.len(), 10);

        let bb = shows.iter().find(|s| s.title.contains("Breaking Bad")).expect("Breaking Bad in DB");
        assert_eq!(bb.year, Some(2008));
        assert_eq!(bb.tvdb_id, Some(81189));
        assert_eq!(bb.has_nfo, 1);
        assert_eq!(bb.has_poster, 1);

        // Check Episodes for Breaking Bad
        let bb_eps = queries::get_episodes_for_show(&conn, bb.id).unwrap();
        assert!(bb_eps.len() >= 60, "Breaking Bad should have ~62 episodes");

        // Verify multi-episode S05E15-E16
        let multi_ep = bb_eps.iter().find(|e| e.season == 5 && e.episode == 15).expect("S05E15 should exist");
        assert_eq!(multi_ep.episode_end, Some(16));
    }

    // Clean up temporary database
    let _ = std::fs::remove_dir_all(&temp_dir);
    println!("✅ All end-to-end scanner and SQLite tests passed successfully!");
}
