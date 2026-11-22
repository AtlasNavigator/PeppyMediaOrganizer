import React, { useState } from "react";
import { useMediaStore } from "../../hooks/useMediaStore";

export const Toolbar: React.FC = () => {
  const activeSection = useMediaStore((s) => s.activeSection);
  const movies = useMediaStore((s) => s.movies);
  const tvShows = useMediaStore((s) => s.tvShows);
  const movieDuplicates = useMediaStore((s) => s.movieDuplicates);
  const episodeDuplicates = useMediaStore((s) => s.episodeDuplicates);
  const viewMode = useMediaStore((s) => s.viewMode);
  const setViewMode = useMediaStore((s) => s.setViewMode);
  const searchQuery = useMediaStore((s) => s.searchQuery);
  const setSearchQuery = useMediaStore((s) => s.setSearchQuery);
  const statusFilter = useMediaStore((s) => s.statusFilter);
  const setStatusFilter = useMediaStore((s) => s.setStatusFilter);
  const isScanning = useMediaStore((s) => s.isScanning);
  const triggerScan = useMediaStore((s) => s.triggerScan);

  const startRenamePlan = useMediaStore((s) => s.startRenamePlan);
  const canUndo = useMediaStore((s) => s.canUndo);
  const undoLastRename = useMediaStore((s) => s.undoLastRename);
  const [isUndoing, setIsUndoing] = useState(false);

  const handleUndo = async () => {
    if (!window.confirm("Are you sure you want to revert the last batch rename operation?")) {
      return;
    }
    try {
      setIsUndoing(true);
      const msg = await undoLastRename();
      alert(`✅ ${msg}`);
    } catch (err) {
      alert(`Undo failed: ${err}`);
    } finally {
      setIsUndoing(false);
    }
  };

  let title = `Movies (${movies.length})`;
  if (activeSection === "tvshows") {
    title = `TV Shows (${tvShows.length})`;
  } else if (activeSection === "duplicates") {
    title = `Duplicates (${movieDuplicates.length + episodeDuplicates.length} groups)`;
  }

  return (
    <header className="win9x-raised p-2 flex items-center justify-between gap-2.5 flex-shrink-0 select-none">
      <div className="flex items-center gap-2">
        <span className="font-bold text-[0.9375rem] tracking-wide">{title}</span>
        <button
          onClick={() => triggerScan()}
          disabled={isScanning}
          className="win9x-button text-[0.875rem] font-bold"
          title="Rescan configured library roots"
        >
          {isScanning ? "⏳ Scanning..." : "⟳ Scan Library"}
        </button>

        <button
          onClick={() => startRenamePlan("batch")}
          className="win9x-button text-[0.875rem] font-bold text-green-900"
          title="Preview dry-run batch rename for library conforming to Jellyfin standards"
        >
          🏷 Batch Rename
        </button>

        <button
          onClick={handleUndo}
          disabled={!canUndo || isUndoing}
          className="win9x-button text-[0.875rem] font-bold text-amber-900"
          title={canUndo ? "Revert the last batch rename operation" : "No previous rename operation to undo"}
        >
          {isUndoing ? "Undoing..." : "↩ Undo Rename"}
        </button>
      </div>

      <div className="flex items-center gap-2.5">
        {/* View Mode Segmented Buttons */}
        {(activeSection === "movies" || activeSection === "tvshows") && (
          <div className="flex items-center win9x-panel-inset p-0.5 bg-[#dfdfdf]">
            <button
              onClick={() => setViewMode("table")}
              className={`px-2.5 py-0.5 text-[0.8125rem] font-bold cursor-pointer border-none outline-none ${
                viewMode === "table"
                  ? "bg-[#000080] text-white"
                  : "bg-transparent text-black hover:bg-[#d0d0d0]"
              }`}
              title="Details Table View (Ctrl+G)"
            >
              ☰ Details
            </button>
            <button
              onClick={() => setViewMode("grid")}
              className={`px-2.5 py-0.5 text-[0.8125rem] font-bold cursor-pointer border-none outline-none ${
                viewMode === "grid"
                  ? "bg-[#000080] text-white"
                  : "bg-transparent text-black hover:bg-[#d0d0d0]"
              }`}
              title="Poster Grid View (Ctrl+G)"
            >
              ⊞ Grid
            </button>
          </div>
        )}

        <label className="flex items-center gap-1.5 text-[0.875rem]">
          <span>Filter:</span>
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="win9x-input cursor-pointer text-[0.875rem]"
          >
            <option value="all">All Items</option>
            <option value="matched">Matched (Has Provider ID)</option>
            <option value="unmatched">Unmatched (Missing IDs)</option>
            <option value="missing_nfo">Missing NFO</option>
            <option value="missing_poster">Missing Poster</option>
          </select>
        </label>

        <div className="flex items-center gap-1">
          <input
            id="search-input"
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search (press / to focus)..."
            className="win9x-input w-56 text-[0.875rem]"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="win9x-button !min-h-[1.5rem] !px-1.5 text-[0.8125rem]"
              title="Clear search"
            >
              ✕
            </button>
          )}
        </div>
      </div>
    </header>
  );
};
