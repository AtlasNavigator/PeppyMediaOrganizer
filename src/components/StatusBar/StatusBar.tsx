import React from "react";
import { useMediaStore } from "../../hooks/useMediaStore";

export const StatusBar: React.FC = () => {
  const statusMessage = useMediaStore((s) => s.statusMessage);
  const isScanning = useMediaStore((s) => s.isScanning);
  const scanProgress = useMediaStore((s) => s.scanProgress);
  const movies = useMediaStore((s) => s.movies);
  const tvShows = useMediaStore((s) => s.tvShows);
  const watcherStatus = useMediaStore((s) => s.watcherStatus);
  const isImdbInstalled = useMediaStore((s) => s.isImdbInstalled);
  const isImdbSyncing = useMediaStore((s) => s.isImdbSyncing);
  const imdbSyncProgress = useMediaStore((s) => s.imdbSyncProgress);
  const startImdbDownload = useMediaStore((s) => s.startImdbDownload);

  let displayMsg = statusMessage;
  if (isScanning && scanProgress) {
    displayMsg = `Scanning (${scanProgress.scanned}/${scanProgress.total_dirs}): ${scanProgress.current_path}`;
  }

  return (
    <footer className="h-8 flex-shrink-0 bg-[#c0c0c0] flex items-center gap-1.5 px-1.5 border-t border-[#808080] select-none text-[0.8125rem]">
      {/* Status Message Segment */}
      <div className="win9x-status-segment h-[1.65rem] flex-1 truncate font-medium">
        {displayMsg}
      </div>

      {/* Watcher Status Segment */}
      <div
        className={`win9x-status-segment h-[1.65rem] w-40 justify-center truncate ${
          watcherStatus?.enabled && (watcherStatus?.watched_roots_count ?? 0) > 0
            ? "text-[#005500] font-bold"
            : "text-[#555555]"
        }`}
        title={
          watcherStatus?.enabled
            ? `Background watcher active on ${watcherStatus.watched_roots_count} library roots`
            : "Background watcher disabled"
        }
      >
        {watcherStatus?.enabled
          ? `👁 Watching (${watcherStatus.watched_roots_count})`
          : "👁 Watching: Off"}
      </div>

      {/* IMDb Dataset Lifecycle Segment */}
      {isImdbSyncing ? (
        <div
          className="win9x-status-segment h-[1.65rem] w-64 text-blue-900 font-bold justify-start px-2 truncate"
          title={imdbSyncProgress || "IMDb sync active"}
        >
          <span className="truncate">⏳ {imdbSyncProgress || "IMDb syncing..."}</span>
        </div>
      ) : isImdbInstalled === false ? (
        <div
          className="win9x-status-segment h-[1.65rem] w-48 text-[#880000] font-medium justify-between px-1.5"
          title="IMDb offline dataset is not installed. Click to download in background."
        >
          <span className="truncate">IMDb: Missing</span>
          <button
            onClick={() => startImdbDownload()}
            className="win9x-button !px-1.5 !py-0 text-[0.75rem] font-bold text-blue-900"
          >
            Download
          </button>
        </div>
      ) : isImdbInstalled === true ? (
        <div
          className="win9x-status-segment h-[1.65rem] w-36 text-[#005500] font-medium justify-center truncate"
          title="IMDb offline dataset is installed and ready"
        >
          ✓ IMDb Ready
        </div>
      ) : (
        <div className="win9x-status-segment h-[1.65rem] w-36 text-gray-500 justify-center truncate">
          IMDb: Checking...
        </div>
      )}

      {/* Library Stats Segment */}
      <div className="win9x-status-segment h-[1.65rem] w-52 justify-center truncate font-mono">
        {movies.length} Movies · {tvShows.length} Shows
      </div>

      {/* Standards Indicator */}
      <div className="win9x-status-segment h-[1.65rem] w-44 justify-center font-bold text-blue-900 whitespace-nowrap">
        Jellyfin Standards
      </div>
    </footer>
  );
};
