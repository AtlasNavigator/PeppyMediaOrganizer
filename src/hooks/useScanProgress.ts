import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ScanProgress, ScanResult, WatcherChangeEvent } from "../lib/types";
import { useMediaStore } from "./useMediaStore";

export function useScanProgress() {
  const setScanProgress = useMediaStore((state) => state.setScanProgress);
  const setStatusMessage = useMediaStore((state) => state.setStatusMessage);
  const loadMovies = useMediaStore((state) => state.loadMovies);
  const loadTvShows = useMediaStore((state) => state.loadTvShows);
  const loadWatcherStatus = useMediaStore((state) => state.loadWatcherStatus);

  useEffect(() => {
    const unlistens: (() => void)[] = [];

    listen<ScanProgress>("scan:progress", (event) => {
      setScanProgress(event.payload);
      setStatusMessage(
        `Scanning [${event.payload.scanned}/${event.payload.total_dirs}]: ${event.payload.current_path}`
      );
    }).then((fn) => unlistens.push(fn));

    listen<ScanResult>("scan:complete", (event) => {
      setStatusMessage(
        `Scan complete: ${event.payload.movies_found} movies, ${event.payload.shows_found} shows (${event.payload.duration_ms}ms)`
      );
      loadMovies();
      loadTvShows();
      loadWatcherStatus();
    }).then((fn) => unlistens.push(fn));

    listen<WatcherChangeEvent>("watcher:change", (event) => {
      setStatusMessage(
        `[Auto-Watch] Detected ${event.payload.paths.length} change(s) in ${event.payload.media_type} library root #${event.payload.root_id}`
      );
      loadMovies();
      loadTvShows();
      loadWatcherStatus();
    }).then((fn) => unlistens.push(fn));

    return () => {
      for (const fn of unlistens) {
        fn();
      }
    };
  }, [setScanProgress, setStatusMessage, loadMovies, loadTvShows, loadWatcherStatus]);
}
