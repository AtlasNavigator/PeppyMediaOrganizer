import { invoke } from "@tauri-apps/api/core";
import type {
  DatasetUpdateInfo,
  Episode,
  EpisodeRemapItem,
  ImdbSyncStatus,
  LibraryRoot,
  Movie,
  MovieSearchResult,
  ProviderConfigDto,
  ProviderIdMap,
  RenameExecutionResult,
  RenamePlan,
  ScanResult,
  TvSearchResult,
  TvShow,
  UndoResult,
  WatcherStatus,
  MediaStreamInfo,
  MovieDuplicateGroup,
  EpisodeDuplicateGroup,
} from "./types";

export const api = {
  // Scanner
  scanLibrary: (rootId: number) =>
    invoke<ScanResult>("scan_library", { rootId }),

  // Media queries
  getMovies: (libraryRootId?: number) =>
    invoke<Movie[]>("get_movies", { libraryRootId: libraryRootId ?? null }),
  getTvShows: (libraryRootId?: number) =>
    invoke<TvShow[]>("get_tv_shows", { libraryRootId: libraryRootId ?? null }),
  getMovieDetail: (id: number) =>
    invoke<Movie | null>("get_movie_detail", { id }),
  getTvShowDetail: (id: number) =>
    invoke<TvShow | null>("get_tv_show_detail", { id }),
  getEpisodes: (tvShowId: number) =>
    invoke<Episode[]>("get_episodes", { tvShowId }),

  // Library & Settings
  addLibraryRoot: (path: string, mediaType: "movie" | "tvshow") =>
    invoke<number>("add_library_root", { path, mediaType }),
  getLibraryRoots: () =>
    invoke<LibraryRoot[]>("get_library_roots"),
  removeLibraryRoot: (id: number) =>
    invoke<void>("remove_library_root", { id }),
  toggleLibraryRootEnabled: (id: number, enabled: boolean) =>
    invoke<void>("toggle_library_root_enabled", { id, enabled }),
  getAppSetting: (key: string) =>
    invoke<string | null>("get_app_setting", { key }),
  setAppSetting: (key: string, value: string) =>
    invoke<void>("set_app_setting", { key, value }),

  // Phase 2: Provider Configurations & Priorities
  getProviderConfigs: () =>
    invoke<ProviderConfigDto[]>("get_provider_configs"),
  setProviderConfig: (
    providerId: string,
    enabled: boolean,
    apiKey: string | null,
    pinOrToken: string | null
  ) =>
    invoke<void>("set_provider_config", {
      providerId,
      enabled,
      apiKey,
      pinOrToken,
    }),
  getFieldPriorities: () =>
    invoke<Record<string, string[]>>("get_field_priorities"),
  setFieldPriority: (fieldName: string, priorityOrder: string[]) =>
    invoke<void>("set_field_priority", {
      fieldName,
      priorityOrder,
    }),

  // Phase 2: Scraping & Matching
  searchMetadataMovie: (title: string, year?: number) =>
    invoke<MovieSearchResult[]>("search_metadata_movie", {
      title,
      year: year ?? null,
    }),
  searchMetadataTv: (title: string, year?: number) =>
    invoke<TvSearchResult[]>("search_metadata_tv", {
      title,
      year: year ?? null,
    }),
  scrapeAndSaveMovie: (movieId: number, providerIds: ProviderIdMap) =>
    invoke<void>("scrape_and_save_movie", {
      movieId,
      providerIds,
    }),
  scrapeAndSaveTv: (tvShowId: number, providerIds: ProviderIdMap, targetOrder: string) =>
    invoke<void>("scrape_and_save_tv", {
      tvShowId,
      providerIds,
      targetOrder,
    }),
  previewEpisodeRemap: (tvShowId: number, targetOrder: string) =>
    invoke<EpisodeRemapItem[]>("preview_episode_remap", {
      tvShowId,
      targetOrder,
    }),

  // Phase 2: IMDb Local Dataset Management
  checkImdbUpdates: () =>
    invoke<DatasetUpdateInfo[]>("check_imdb_updates"),
  getImdbSyncStatus: () =>
    invoke<ImdbSyncStatus[]>("get_imdb_sync_status"),
  indexImdbFromLocal: (folderPath: string) =>
    invoke<string>("index_imdb_from_local", {
      folderPath,
    }),

  // Phase 3: Renamer & Reversible Undo
  planRenameMovie: (movieId: number) =>
    invoke<RenamePlan>("plan_rename_movie", { movieId }),
  planRenameTvShow: (tvShowId: number) =>
    invoke<RenamePlan>("plan_rename_tv_show", { tvShowId }),
  planBatchRename: (movieIds: number[], tvShowIds: number[]) =>
    invoke<RenamePlan>("plan_batch_rename", { movieIds, tvShowIds }),
  planResolveDuplicates: (
    keepFilePath: string,
    obsoleteFilePaths: string[],
    mediaType: "movie" | "episode"
  ) =>
    invoke<RenamePlan>("plan_resolve_duplicates", {
      keepFilePath,
      obsoleteFilePaths,
      mediaType,
    }),
  executeRenamePlan: (plan: RenamePlan, selectedActionIds?: string[]) =>
    invoke<RenameExecutionResult>("execute_rename_plan", {
      plan,
      selectedActionIds: selectedActionIds ?? null,
    }),
  undoLastRename: () =>
    invoke<UndoResult>("undo_last_rename"),
  canUndo: () =>
    invoke<boolean>("can_undo"),

  // Phase 4: File Watcher
  getWatcherStatus: () =>
    invoke<WatcherStatus>("get_watcher_status"),
  setWatcherEnabled: (enabled: boolean) =>
    invoke<WatcherStatus>("set_watcher_enabled", { enabled }),
  reloadWatchedRoots: () =>
    invoke<WatcherStatus>("reload_watched_roots"),

  // Phase 5: Media Streams & Technical Info
  getMediaStreamInfo: (mediaType: "movie" | "episode", mediaId: number) =>
    invoke<MediaStreamInfo | null>("get_media_stream_info", {
      mediaType,
      mediaId,
    }),
  probeMediaFile: (filePath: string, mediaType: "movie" | "episode", mediaId: number) =>
    invoke<MediaStreamInfo>("probe_media_file", {
      filePath,
      mediaType,
      mediaId,
    }),
  probePendingBatch: (batchSize?: number) =>
    invoke<number>("probe_pending_batch", {
      batchSize: batchSize ?? null,
    }),
  getFfprobePath: () =>
    invoke<string | null>("get_ffprobe_path"),
  setFfprobePath: (path: string) =>
    invoke<void>("set_ffprobe_path", { path }),

  // Phase 5: Duplicate Detection
  getMovieDuplicates: () =>
    invoke<MovieDuplicateGroup[]>("get_movie_duplicates"),
  getEpisodeDuplicates: () =>
    invoke<EpisodeDuplicateGroup[]>("get_episode_duplicates"),

  // UI Display Scaling & Zoom
  getSystemTextScale: () =>
    invoke<number>("get_system_text_scale"),
  setWindowZoom: (zoom: number) =>
    invoke<void>("set_window_zoom", { zoom }),
};
