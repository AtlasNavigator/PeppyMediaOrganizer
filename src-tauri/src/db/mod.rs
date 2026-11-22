pub mod migrations;
pub mod models;
pub mod queries;

use std::path::Path;
use rusqlite::Connection;

pub fn init_db(app_data_dir: &Path) -> Result<Connection, rusqlite::Error> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("media.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrations::run_migrations(&conn)?;
    Ok(conn)
}
