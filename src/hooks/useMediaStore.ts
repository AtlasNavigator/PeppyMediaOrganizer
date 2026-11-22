import { create } from "zustand";
import type {
  Episode,
  EpisodeDuplicateGroup,
  LibraryRoot,
  MediaStreamInfo,
  Movie,
  MovieDuplicateGroup,
  RenamePlan,
  ScanProgress,
  TvShow,
  WatcherStatus,
} from "../lib/types";
import { api } from "../lib/tauri";

export type Section = "movies" | "tvshows" | "duplicates" | "settings";

interface MediaStore {
  // Navigation
  activeSection: Section;
  setActiveSection: (section: Section) => void;

  // View Mode (Table vs. Poster Grid)
  viewMode: "table" | "grid";
  setViewMode: (mode: "table" | "grid") => void;
  toggleViewMode: () => void;

  // Multi-Selection State
  selectedMovieIds: Set<number>;
  selectedShowIds: Set<number>;
  anchorId: number | null;
  toggleItemSelection: (id: number) => void;
  selectItemRange: (id: number, orderedIds: number[]) => void;
  selectAllFiltered: (orderedIds: number[]) => void;
  clearMultiSelection: () => void;

  // Movies
  movies: Movie[];
  selectedMovieId: number | null;
  loadMovies: () => Promise<void>;
  selectMovie: (id: number | null) => void;

  // TV Shows & Episodes
  tvShows: TvShow[];
  selectedShowId: number | null;
  episodes: Episode[];
  loadTvShows: () => Promise<void>;
  selectShow: (id: number | null) => void;
  loadEpisodes: (showId: number) => Promise<void>;

  // Library Roots
  libraryRoots: LibraryRoot[];
  loadLibraryRoots: () => Promise<void>;

  // Phase 4: Watcher State
  watcherStatus: WatcherStatus | null;
  loadWatcherStatus: () => Promise<void>;
  setWatcherEnabled: (enabled: boolean) => Promise<void>;

  // Phase 5: Technical Media Stream Info
  activeStreamInfo: MediaStreamInfo | null;
  isLoadingStreamInfo: boolean;
  loadStreamInfo: (mediaType: "movie" | "episode", mediaId: number) => Promise<void>;
  probeCurrentItem: (filePath: string, mediaType: "movie" | "episode", mediaId: number) => Promise<void>;
  runPendingProbes: () => Promise<void>;

  // Phase 5: Duplicate Detection
  movieDuplicates: MovieDuplicateGroup[];
  episodeDuplicates: EpisodeDuplicateGroup[];
  isLoadingDuplicates: boolean;
  loadDuplicates: () => Promise<void>;

  // Scanner State
  isScanning: boolean;
  scanProgress: ScanProgress | null;
  statusMessage: string;
  setScanning: (scanning: boolean) => void;
  setScanProgress: (progress: ScanProgress | null) => void;
  setStatusMessage: (msg: string) => void;
  triggerScan: (rootId?: number) => Promise<void>;

  // Search & Filter
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  statusFilter: string;
  setStatusFilter: (filter: string) => void;

  // Phase 3: Renamer & Undo
  isRenameModalOpen: boolean;
  renamePlan: RenamePlan | null;
  isLoadingRenamePlan: boolean;
  canUndo: boolean;
  openRenameModal: (plan: RenamePlan) => void;
  closeRenameModal: () => void;
  startRenamePlan: (type: "movie" | "tvshow" | "batch", id?: number, customIds?: number[]) => Promise<void>;
  resolveDuplicatesKeepSingle: (keepFilePath: string, obsoleteFilePaths: string[], mediaType: "movie" | "episode") => Promise<void>;
  checkCanUndo: () => Promise<void>;
  undoLastRename: () => Promise<string>;

  // Display Scaling & Zoom
  uiScale: number;
  systemTextScale: number;
  initUiScale: () => Promise<void>;
  setUiScale: (scale: number) => Promise<void>;
}

export const useMediaStore = create<MediaStore>((set, get) => ({
  activeSection: "movies",
  setActiveSection: (section) => {
    set({ activeSection: section });
    if (section === "duplicates") {
      get().loadDuplicates();
    }
  },

  // View Mode
  viewMode: "table",
  setViewMode: (mode) => set({ viewMode: mode }),
  toggleViewMode: () =>
    set((s) => ({ viewMode: s.viewMode === "table" ? "grid" : "table" })),

  // Multi-Selection State
  selectedMovieIds: new Set<number>(),
  selectedShowIds: new Set<number>(),
  anchorId: null,

  toggleItemSelection: (id) => {
    const { activeSection, selectedMovieIds, selectedShowIds } = get();
    if (activeSection === "movies") {
      const next = new Set(selectedMovieIds);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      set({
        selectedMovieIds: next,
        selectedMovieId: next.has(id) ? id : (Array.from(next)[0] ?? null),
        anchorId: id,
      });
      if (next.has(id)) {
        get().loadStreamInfo("movie", id);
      }
    } else {
      const next = new Set(selectedShowIds);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      set({
        selectedShowIds: next,
        selectedShowId: next.has(id) ? id : (Array.from(next)[0] ?? null),
        anchorId: id,
      });
    }
  },

  selectItemRange: (id, orderedIds) => {
    const { activeSection, anchorId, selectedMovieIds, selectedShowIds } = get();
    if (!anchorId || orderedIds.length === 0) {
      if (activeSection === "movies") {
        set({ selectedMovieIds: new Set([id]), selectedMovieId: id, anchorId: id });
        get().loadStreamInfo("movie", id);
      } else {
        set({ selectedShowIds: new Set([id]), selectedShowId: id, anchorId: id });
      }
      return;
    }

    const anchorIdx = orderedIds.indexOf(anchorId);
    const targetIdx = orderedIds.indexOf(id);
    if (anchorIdx === -1 || targetIdx === -1) return;

    const start = Math.min(anchorIdx, targetIdx);
    const end = Math.max(anchorIdx, targetIdx);
    const range = orderedIds.slice(start, end + 1);

    if (activeSection === "movies") {
      const next = new Set(selectedMovieIds);
      for (const item of range) next.add(item);
      set({ selectedMovieIds: next, selectedMovieId: id });
      get().loadStreamInfo("movie", id);
    } else {
      const next = new Set(selectedShowIds);
      for (const item of range) next.add(item);
      set({ selectedShowIds: next, selectedShowId: id });
    }
  },

  selectAllFiltered: (orderedIds) => {
    const { activeSection } = get();
    if (activeSection === "movies") {
      set({
        selectedMovieIds: new Set(orderedIds),
        selectedMovieId: orderedIds[0] ?? null,
      });
      if (orderedIds[0]) get().loadStreamInfo("movie", orderedIds[0]);
    } else {
      set({
        selectedShowIds: new Set(orderedIds),
        selectedShowId: orderedIds[0] ?? null,
      });
    }
  },

  clearMultiSelection: () => {
    const { activeSection, selectedMovieId, selectedShowId } = get();
    if (activeSection === "movies") {
      set({ selectedMovieIds: selectedMovieId !== null ? new Set([selectedMovieId]) : new Set() });
    } else {
      set({ selectedShowIds: selectedShowId !== null ? new Set([selectedShowId]) : new Set() });
    }
  },

  // Movies
  movies: [],
  selectedMovieId: null,
  loadMovies: async () => {
    try {
      const movies = await api.getMovies();
      set({ movies });
      if (movies.length > 0 && get().selectedMovieId === null) {
        const firstId = movies[0].id;
        set({
          selectedMovieId: firstId,
          selectedMovieIds: new Set([firstId]),
          anchorId: firstId,
        });
        get().loadStreamInfo("movie", firstId);
      }
    } catch (err) {
      console.error("Failed to load movies:", err);
    }
  },
  selectMovie: (id) => {
    set({
      selectedMovieId: id,
      selectedMovieIds: id !== null ? new Set([id]) : new Set(),
      anchorId: id,
    });
    if (id !== null) {
      get().loadStreamInfo("movie", id);
    } else {
      set({ activeStreamInfo: null });
    }
  },

  // TV Shows
  tvShows: [],
  selectedShowId: null,
  episodes: [],
  loadTvShows: async () => {
    try {
      const tvShows = await api.getTvShows();
      set({ tvShows });
      if (tvShows.length > 0 && get().selectedShowId === null) {
        const firstShowId = tvShows[0].id;
        set({
          selectedShowId: firstShowId,
          selectedShowIds: new Set([firstShowId]),
          anchorId: firstShowId,
        });
        get().loadEpisodes(firstShowId);
      }
    } catch (err) {
      console.error("Failed to load TV shows:", err);
    }
  },
  selectShow: (id) => {
    set({
      selectedShowId: id,
      selectedShowIds: id !== null ? new Set([id]) : new Set(),
      anchorId: id,
    });
    if (id !== null) {
      get().loadEpisodes(id);
    } else {
      set({ episodes: [], activeStreamInfo: null });
    }
  },
  loadEpisodes: async (showId) => {
    try {
      const episodes = await api.getEpisodes(showId);
      set({ episodes });
    } catch (err) {
      console.error("Failed to load episodes for show", showId, err);
    }
  },

  // Library Roots
  libraryRoots: [],
  loadLibraryRoots: async () => {
    try {
      const libraryRoots = await api.getLibraryRoots();
      set({ libraryRoots });
      // Keep watcher roots in sync
      const status = await api.reloadWatchedRoots().catch(() => null);
      if (status) {
        set({ watcherStatus: status });
      }
    } catch (err) {
      console.error("Failed to load library roots:", err);
    }
  },

  // Phase 4: Watcher
  watcherStatus: null,
  loadWatcherStatus: async () => {
    try {
      const watcherStatus = await api.getWatcherStatus();
      set({ watcherStatus });
    } catch (err) {
      console.error("Failed to load watcher status:", err);
    }
  },
  setWatcherEnabled: async (enabled: boolean) => {
    try {
      const watcherStatus = await api.setWatcherEnabled(enabled);
      set({ watcherStatus });
      set({
        statusMessage: enabled
          ? `Background library watching enabled (${watcherStatus.watched_roots_count} roots)`
          : "Background library watching disabled",
      });
    } catch (err) {
      console.error("Failed to set watcher enabled:", err);
    }
  },

  // Scanner
  isScanning: false,
  scanProgress: null,
  statusMessage: "Ready",
  setScanning: (isScanning) => set({ isScanning }),
  setScanProgress: (scanProgress) => set({ scanProgress }),
  setStatusMessage: (statusMessage) => set({ statusMessage }),

  triggerScan: async (rootId) => {
    const { libraryRoots, loadMovies, loadTvShows, loadLibraryRoots } = get();
    const rootsToScan = rootId
      ? libraryRoots.filter((r) => r.id === rootId)
      : libraryRoots.filter((r) => r.enabled === 1);

    if (rootsToScan.length === 0) {
      set({ statusMessage: "No library configured. Add a library folder in Settings." });
      return;
    }

    set({ isScanning: true, statusMessage: "Starting scan..." });

    for (const root of rootsToScan) {
      try {
        set({ statusMessage: `Scanning ${root.path}...` });
        const res = await api.scanLibrary(root.id);
        set({
          statusMessage: `Scanned ${root.path}: ${res.movies_found} movies, ${res.shows_found} shows (${res.duration_ms}ms)`,
        });
      } catch (err) {
        console.error("Error scanning library:", err);
        set({ statusMessage: `Error scanning ${root.path}: ${err}` });
      }
    }

    await Promise.all([loadMovies(), loadTvShows(), loadLibraryRoots()]);
    set({ isScanning: false, scanProgress: null });
  },

  // Search & Filter
  searchQuery: "",
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  statusFilter: "all",
  setStatusFilter: (statusFilter) => set({ statusFilter }),

  // Phase 5: Technical Media Stream Info
  activeStreamInfo: null,
  isLoadingStreamInfo: false,
  loadStreamInfo: async (mediaType, mediaId) => {
    try {
      set({ isLoadingStreamInfo: true });
      const info = await api.getMediaStreamInfo(mediaType, mediaId);
      set({ activeStreamInfo: info, isLoadingStreamInfo: false });
    } catch (err) {
      console.error("Failed to load stream info:", err);
      set({ activeStreamInfo: null, isLoadingStreamInfo: false });
    }
  },
  probeCurrentItem: async (filePath, mediaType, mediaId) => {
    try {
      set({ isLoadingStreamInfo: true });
      const info = await api.probeMediaFile(filePath, mediaType, mediaId);
      set({ activeStreamInfo: info, isLoadingStreamInfo: false });
    } catch (err) {
      console.error("Failed to probe file:", err);
      set({ isLoadingStreamInfo: false });
      alert(`Probe failed: ${err}`);
    }
  },
  runPendingProbes: async () => {
    try {
      const count = await api.probePendingBatch(15);
      if (count > 0) {
        const { activeSection, selectedMovieId } = get();
        if (activeSection === "movies" && selectedMovieId) {
          get().loadStreamInfo("movie", selectedMovieId);
        }
      }
    } catch (err) {
      console.error("Background probe error:", err);
    }
  },

  // Phase 5: Duplicate Detection
  movieDuplicates: [],
  episodeDuplicates: [],
  isLoadingDuplicates: false,
  loadDuplicates: async () => {
    try {
      set({ isLoadingDuplicates: true });
      const [movies, episodes] = await Promise.all([
        api.getMovieDuplicates(),
        api.getEpisodeDuplicates(),
      ]);
      set({
        movieDuplicates: movies,
        episodeDuplicates: episodes,
        isLoadingDuplicates: false,
      });
    } catch (err) {
      console.error("Failed to load duplicates:", err);
      set({ isLoadingDuplicates: false });
    }
  },

  // Phase 3: Renamer & Undo
  isRenameModalOpen: false,
  renamePlan: null,
  isLoadingRenamePlan: false,
  canUndo: false,

  openRenameModal: (plan) => set({ isRenameModalOpen: true, renamePlan: plan }),
  closeRenameModal: () => set({ isRenameModalOpen: false, renamePlan: null }),

  startRenamePlan: async (type, id, customIds) => {
    set({ isRenameModalOpen: true, isLoadingRenamePlan: true, renamePlan: null });
    try {
      let plan: RenamePlan;
      if (type === "movie" && id !== undefined) {
        plan = await api.planRenameMovie(id);
      } else if (type === "tvshow" && id !== undefined) {
        plan = await api.planRenameTvShow(id);
      } else if (customIds && customIds.length > 0) {
        const { activeSection } = get();
        if (activeSection === "movies") {
          plan = await api.planBatchRename(customIds, []);
        } else {
          plan = await api.planBatchRename([], customIds);
        }
      } else {
        plan = await api.planBatchRename([], []);
      }
      set({ renamePlan: plan, isLoadingRenamePlan: false });
    } catch (err) {
      console.error("Failed to generate rename plan:", err);
      set({ isLoadingRenamePlan: false });
    }
  },

  resolveDuplicatesKeepSingle: async (keepFilePath, obsoleteFilePaths, mediaType) => {
    try {
      set({ isLoadingRenamePlan: true, isRenameModalOpen: true });
      const plan = await api.planResolveDuplicates(keepFilePath, obsoleteFilePaths, mediaType);
      set({ renamePlan: plan, isLoadingRenamePlan: false });
    } catch (err) {
      console.error("Failed to plan duplicate resolution:", err);
      set({ isLoadingRenamePlan: false, isRenameModalOpen: false });
      alert(`Failed to plan duplicate resolution: ${err}`);
    }
  },

  checkCanUndo: async () => {
    try {
      const can = await api.canUndo();
      set({ canUndo: can });
    } catch (err) {
      console.error("Error checking canUndo:", err);
    }
  },

  undoLastRename: async () => {
    const { loadMovies, loadTvShows, loadDuplicates, checkCanUndo } = get();
    try {
      const res = await api.undoLastRename();
      await Promise.all([loadMovies(), loadTvShows(), loadDuplicates(), checkCanUndo()]);
      const msg = `Reverted ${res.reverted_count} files back to their previous locations!`;
      set({ statusMessage: msg });
      return msg;
    } catch (err) {
      console.error("Undo error:", err);
      throw err;
    }
  },

  // Display Scaling & Zoom
  uiScale: 1.0,
  systemTextScale: 1.0,

  initUiScale: async () => {
    try {
      let sysScale = 1.0;
      try {
        sysScale = await api.getSystemTextScale();
      } catch (err) {
        console.warn("Could not query system text scale:", err);
      }
      set({ systemTextScale: sysScale });

      // Check saved user preference
      const saved = await api.getAppSetting("ui_scale");
      if (saved) {
        const parsed = parseFloat(saved);
        if (!isNaN(parsed) && parsed >= 0.5 && parsed <= 3.0) {
          set({ uiScale: parsed });
          await api.setWindowZoom(parsed);
          return;
        }
      }

      // If no explicit preference saved, auto-apply Linux system text scale if devicePixelRatio == 1
      if (typeof window !== "undefined" && window.devicePixelRatio === 1 && sysScale > 1.05) {
        set({ uiScale: sysScale });
        await api.setWindowZoom(sysScale);
      }
    } catch (err) {
      console.error("initUiScale error:", err);
    }
  },

  setUiScale: async (scale: number) => {
    const clamped = Math.round(Math.min(2.5, Math.max(0.7, scale)) * 100) / 100;
    set({ uiScale: clamped });
    try {
      await api.setWindowZoom(clamped);
      await api.setAppSetting("ui_scale", clamped.toString());
    } catch (err) {
      console.error("Failed to set window zoom:", err);
    }
  },
}));
