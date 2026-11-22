use serde::{Deserialize, Serialize};
use crate::db::models::Episode;
use crate::providers::{EpisodeOrder, TvShowDetails};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeRemapItem {
    pub episode_id: i64,
    pub current_file_path: String,
    pub current_season: i32,
    pub current_episode: i32,
    pub current_title: Option<String>,
    pub target_season: i32,
    pub target_episode: i32,
    pub target_title: Option<String>,
    pub target_order: EpisodeOrder,
    pub has_changed: bool,
}

/// Calculates an episode remapping preview between local filesystem episodes
/// and provider episode ordering (Aired, DVD, Streaming, Absolute).
pub fn calculate_episode_remap(
    local_episodes: &[Episode],
    target_order: EpisodeOrder,
    show_details: &TvShowDetails,
) -> Vec<EpisodeRemapItem> {
    let mut remap_list = Vec::new();

    for local in local_episodes {
        // Try matching by title first (fuzzy/exact), then by air date or index
        let matched_ep = if let Some(ref local_title) = local.title {
            let clean_local = local_title.trim().to_lowercase();
            show_details.episodes.iter().find(|e| {
                if let Some(ref p_title) = e.title {
                    p_title.trim().to_lowercase() == clean_local
                } else {
                    false
                }
            })
        } else {
            None
        };

        // Fallback to match by season & episode in provider list
        let target_match = matched_ep.or_else(|| {
            show_details
                .episodes
                .iter()
                .find(|e| e.season == local.season as u32 && e.episode == local.episode as u32)
        });

        if let Some(target) = target_match {
            let has_changed = target.season as i32 != local.season || target.episode as i32 != local.episode;

            remap_list.push(EpisodeRemapItem {
                episode_id: local.id,
                current_file_path: local.file_path.clone(),
                current_season: local.season,
                current_episode: local.episode,
                current_title: local.title.clone(),
                target_season: target.season as i32,
                target_episode: target.episode as i32,
                target_title: target.title.clone(),
                target_order,
                has_changed,
            });
        } else {
            // Unmatched item retains current positions
            remap_list.push(EpisodeRemapItem {
                episode_id: local.id,
                current_file_path: local.file_path.clone(),
                current_season: local.season,
                current_episode: local.episode,
                current_title: local.title.clone(),
                target_season: local.season,
                target_episode: local.episode,
                target_title: local.title.clone(),
                target_order,
                has_changed: false,
            });
        }
    }

    remap_list
}
