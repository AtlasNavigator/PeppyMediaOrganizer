pub mod parser;
pub mod walker;
pub mod watcher;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub scanned: usize,
    pub total_dirs: usize,
    pub current_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub root_id: i64,
    pub movies_found: usize,
    pub shows_found: usize,
    pub episodes_found: usize,
    pub duration_ms: u64,
}
