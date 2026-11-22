export interface LibraryRoot {
  id: number;
  path: string;
  media_type: "movie" | "tvshow";
  enabled: number;
  last_scanned_at: string | null;
  created_at: string;
}

export interface Movie {
  id: number;
  library_root_id: number;
  folder_path: string;
  file_path: string;
  title: string;
  sort_title: string | null;
  year: number | null;
  resolution: string | null;
  imdb_id: string | null;
  tmdb_id: number | null;
  has_nfo: number;
  has_poster: number;
  has_backdrop: number;
  file_size: number | null;
  subtitle_count: number;
  version_label: string | null;
  scan_status: "pending" | "matched" | "unmatched" | "error";
  created_at: string;
  updated_at: string;
  // Phase 2 Metadata
  plot?: string | null;
  tagline?: string | null;
  mpaa?: string | null;
  runtime?: number | null;
  genres?: string | null;
  studio?: string | null;
  director?: string | null;
  writers?: string | null;
  actors_json?: string | null;
  poster_url?: string | null;
  backdrop_url?: string | null;
  community_rating?: number | null;
  vote_count?: number | null;
}

export interface TvShow {
  id: number;
  library_root_id: number;
  folder_path: string;
  title: string;
  sort_title: string | null;
  year: number | null;
  tvdb_id: number | null;
  tmdb_id: number | null;
  imdb_id: string | null;
  status: string | null;
  has_nfo: number;
  has_poster: number;
  has_backdrop: number;
  season_count: number;
  episode_count: number;
  scan_status: string;
  created_at: string;
  updated_at: string;
  // Phase 2 Metadata
  plot?: string | null;
  mpaa?: string | null;
  studio?: string | null;
  genres?: string | null;
  poster_url?: string | null;
  backdrop_url?: string | null;
  banner_url?: string | null;
  episode_order_mode?: "aired" | "dvd" | "streaming" | "absolute";
  community_rating?: number | null;
  vote_count?: number | null;
}

export interface Episode {
  id: number;
  tv_show_id: number;
  file_path: string;
  title: string | null;
  season: number;
  episode: number;
  episode_end: number | null;
  resolution: string | null;
  imdb_id: string | null;
  tvdb_id: number | null;
  has_nfo: number;
  file_size: number | null;
  subtitle_count: number;
  scan_status: string;
  created_at: string;
  updated_at: string;
  // Phase 2
  plot?: string | null;
  aired_date?: string | null;
  runtime?: number | null;
  director?: string | null;
  writer?: string | null;
  still_url?: string | null;
  community_rating?: number | null;
  vote_count?: number | null;
}

export interface ScanProgress {
  scanned: number;
  total_dirs: number;
  current_path: string;
}

export interface ScanResult {
  root_id: number;
  movies_found: number;
  shows_found: number;
  episodes_found: number;
  duration_ms: number;
}

// Phase 2: Provider & Scraping Types
export interface ProviderConfigDto {
  provider_id: "imdb" | "tmdb" | "tvdb" | "omdb";
  enabled: boolean;
  api_key: string | null;
  pin_or_token: string | null;
  rate_limit_per_sec: number;
}

export interface ProviderIdMap {
  imdb_id: string | null;
  tmdb_id: number | null;
  tvdb_id: number | null;
}

export interface MovieSearchResult {
  provider_ids: ProviderIdMap;
  title: string;
  year: number | null;
  poster_url: string | null;
  overview: string | null;
  rating: number | null;
  vote_count: number | null;
}

export interface TvSearchResult {
  provider_ids: ProviderIdMap;
  title: string;
  year: number | null;
  poster_url: string | null;
  overview: string | null;
  rating: number | null;
  vote_count: number | null;
}

export interface EpisodeRemapItem {
  episode_id: number;
  current_file_path: string;
  current_season: number;
  current_episode: number;
  current_title: string | null;
  target_season: number;
  target_episode: number;
  target_title: string | null;
  target_order: "aired" | "dvd" | "streaming" | "absolute";
  has_changed: boolean;
}

export interface ImdbSyncStatus {
  dataset_name: string;
  last_modified_header: string | null;
  last_synced_at: string | null;
  record_count: number;
  status: "idle" | "downloading" | "indexing" | "ready" | "error";
  error_message: string | null;
}

export interface DatasetUpdateInfo {
  dataset_name: string;
  upstream_last_modified: string;
  local_last_synced: string | null;
  update_available: boolean;
}

// Phase 3: Renamer & Undo Types
export type RenameItemType =
  | "movie_folder"
  | "movie_video"
  | "tv_show_folder"
  | "season_folder"
  | "episode_video"
  | "subtitle"
  | "nfo"
  | "artwork"
  | "extra";

export type ActionStatus = "will_rename" | "skipped" | "conflict";

export interface RenameAction {
  id: string;
  media_type: "movie" | "tvshow";
  media_id: number;
  item_type: RenameItemType;
  old_path: string;
  new_path: string;
  status: ActionStatus;
  reason: string | null;
}

export interface RenamePlan {
  batch_id: string;
  actions: RenameAction[];
  total_count: number;
  will_rename_count: number;
  skipped_count: number;
  conflict_count: number;
}

export interface RenameExecutionResult {
  batch_id: string;
  executed_count: number;
  failed_count: number;
  errors: string[];
}

export interface UndoResult {
  batch_id: string;
  reverted_count: number;
  failed_count: number;
  errors: string[];
}

export interface WatcherStatus {
  enabled: boolean;
  watched_roots_count: number;
  watched_paths: string[];
  last_event_at: string | null;
}

export interface WatcherChangeEvent {
  root_id: number;
  media_type: "movie" | "tvshow";
  paths: string[];
  timestamp: string;
}

// Phase 5: Media Streams & Technical Info
export interface MediaStreamInfo {
  id: number;
  media_type: "movie" | "episode";
  media_id: number;
  file_path: string;
  container: string | null;
  video_codec: string | null;
  width: number | null;
  height: number | null;
  aspect_ratio: string | null;
  hdr_format: string | null;
  audio_codec: string | null;
  audio_channels: number | null;
  audio_layout: string | null;
  audio_languages: string | null;
  subtitle_languages: string | null;
  bitrate_kbps: number | null;
  duration_sec: number | null;
  probe_status: "pending" | "probed" | "failed";
  probed_at: string | null;
}

// Phase 5: Duplicate Detection Groups
export interface DuplicateMovieItem {
  id: number;
  library_root_id: number;
  folder_path: string;
  file_path: string;
  title: string;
  year: number | null;
  resolution: string | null;
  imdb_id: string | null;
  tmdb_id: number | null;
  file_size: number | null;
  version_label: string | null;
  video_codec: string | null;
  audio_codec: string | null;
  bitrate_kbps: number | null;
}

export interface MovieDuplicateGroup {
  match_key: string;
  reason: string;
  items: DuplicateMovieItem[];
}

export interface DuplicateEpisodeItem {
  id: number;
  tv_show_id: number;
  show_title: string;
  file_path: string;
  season: number;
  episode: number;
  title: string | null;
  resolution: string | null;
  file_size: number | null;
  video_codec: string | null;
  audio_codec: string | null;
  bitrate_kbps: number | null;
}

export interface EpisodeDuplicateGroup {
  tv_show_id: number;
  show_title: string;
  season: number;
  episode: number;
  match_key: string;
  items: DuplicateEpisodeItem[];
}
