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
