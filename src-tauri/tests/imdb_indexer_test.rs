use peppy_media_organizer::providers::imdb::db::init_imdb_db;
use peppy_media_organizer::providers::imdb::indexer::index_ratings_tsv_gz;

#[test]
fn test_imdb_local_dataset_indexing_and_search() {
    let workspace_root = std::env::current_dir().unwrap().parent().unwrap().to_path_buf();
    let basics_path = workspace_root.join("title.basics.tsv.gz");
    let ratings_path = workspace_root.join("title.ratings.tsv.gz");

    if !basics_path.exists() || !ratings_path.exists() {
        println!("Skipping test: title.*.tsv.gz files not present in workspace");
        return;
    }

    let temp_dir = std::env::temp_dir().join(format!("imdb_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("imdb_test.db");

    let mut conn = init_imdb_db(&db_path).expect("Failed to init imdb db");

    println!("Indexing ratings first (small file, 8.4MB)...");
    let ratings_count = index_ratings_tsv_gz(&mut conn, &ratings_path, None)
        .expect("Failed to index ratings");
    println!("Indexed {} ratings", ratings_count);
    assert!(ratings_count > 100_000, "Should index over 100,000 ratings");

    // Test direct tconst rating query
    let mut stmt = conn.prepare("SELECT average_rating, num_votes FROM imdb_ratings WHERE tconst = 'tt0078748'").unwrap();
    let alien_rating: (f64, i64) = stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    println!("Alien (tt0078748) rating in IMDb: {} ({} votes)", alien_rating.0, alien_rating.1);
    assert!(alien_rating.0 >= 8.0, "Alien rating should be >= 8.0");

    let _ = std::fs::remove_dir_all(&temp_dir);
    println!("✅ IMDb dataset test passed successfully!");
}

#[test]
fn test_imdb_installation_and_staleness_lifecycle() {
    use peppy_media_organizer::providers::imdb::downloader::{
        clean_temp_dir, is_imdb_installed, is_imdb_stale,
    };

    let temp_dir = std::env::temp_dir().join(format!("imdb_lifecycle_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("imdb.db");

    let conn = init_imdb_db(&db_path).expect("Failed to init imdb db");

    // 1. Initial empty state: not installed, considered stale
    assert!(!is_imdb_installed(&conn), "Empty DB must report not installed");
    assert!(is_imdb_stale(&conn), "Empty DB must report stale");

    // 2. Insert mock title to simulate installed state
    conn.execute(
        "INSERT INTO imdb_titles (tconst, title_type, primary_title) VALUES ('tt0000001', 'movie', 'Carmencita')",
        [],
    )
    .unwrap();
    assert!(is_imdb_installed(&conn), "Populated DB must report installed");

    // 3. Stale if sync record is older than 7 days (e.g. 10 days ago)
    conn.execute(
        r#"
        INSERT INTO imdb_sync_status (dataset_name, last_synced_at, record_count, status)
        VALUES ('title.basics', datetime('now', '-10 days'), 1, 'ready')
        "#,
        [],
    )
    .unwrap();
    assert!(is_imdb_stale(&conn), "Dataset older than 7 days must report stale");

    // 4. Fresh if sync record is recent (e.g. 2 days ago)
    conn.execute(
        r#"
        UPDATE imdb_sync_status
        SET last_synced_at = datetime('now', '-2 days')
        WHERE dataset_name = 'title.basics'
        "#,
        [],
    )
    .unwrap();
    assert!(!is_imdb_stale(&conn), "Dataset synced 2 days ago must report not stale");

    // 5. Clean temporary directory in app_data_dir
    let mock_temp_dir = temp_dir.join("imdb_temp");
    std::fs::create_dir_all(&mock_temp_dir).unwrap();
    let mock_archive = mock_temp_dir.join("title.basics.tsv.gz");
    std::fs::write(&mock_archive, b"mock compressed data").unwrap();
    assert!(mock_archive.exists(), "Mock archive should exist before clean_temp_dir");

    clean_temp_dir(&temp_dir);
    assert!(!mock_temp_dir.exists(), "clean_temp_dir must delete imdb_temp completely");
    assert!(!mock_archive.exists(), "clean_temp_dir must remove temporary .tsv.gz archives");

    let _ = std::fs::remove_dir_all(&temp_dir);
    println!("✅ IMDb installation, staleness, and temp cleanup lifecycle test passed!");
}
