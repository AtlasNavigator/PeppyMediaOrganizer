use rusqlite::Connection;

use crate::db::models::{EpisodeDuplicateGroup, MovieDuplicateGroup};
use crate::db::queries::{find_episode_duplicates, find_movie_duplicates};

pub fn get_movie_duplicates(conn: &Connection) -> Result<Vec<MovieDuplicateGroup>, String> {
    find_movie_duplicates(conn).map_err(|e| e.to_string())
}

pub fn get_episode_duplicates(conn: &Connection) -> Result<Vec<EpisodeDuplicateGroup>, String> {
    find_episode_duplicates(conn).map_err(|e| e.to_string())
}
