import React, { useState } from "react";
import { useMediaStore } from "../../hooks/useMediaStore";
import { api } from "../../lib/tauri";

export const BatchBar: React.FC = () => {
  const activeSection = useMediaStore((s) => s.activeSection);
  const selectedMovieIds = useMediaStore((s) => s.selectedMovieIds);
  const selectedShowIds = useMediaStore((s) => s.selectedShowIds);
  const clearMultiSelection = useMediaStore((s) => s.clearMultiSelection);
  const startRenamePlan = useMediaStore((s) => s.startRenamePlan);
  const setStatusMessage = useMediaStore((s) => s.setStatusMessage);
  const loadMovies = useMediaStore((s) => s.loadMovies);
  const loadTvShows = useMediaStore((s) => s.loadTvShows);

  const [isProcessing, setIsProcessing] = useState(false);

  const selectedIds = Array.from(
    activeSection === "movies" ? selectedMovieIds : selectedShowIds
  );

  if (selectedIds.length <= 1) {
    return null;
  }

  const handleBatchRename = () => {
    startRenamePlan("batch", undefined, selectedIds);
  };

  const handleBatchScrape = async () => {
    setIsProcessing(true);
    setStatusMessage(`Batch scraping metadata for ${selectedIds.length} items...`);
    let successCount = 0;

    for (const id of selectedIds) {
      try {
        if (activeSection === "movies") {
          const res = await api.searchMetadataMovie(
            useMediaStore.getState().movies.find((m) => m.id === id)?.title || "",
            useMediaStore.getState().movies.find((m) => m.id === id)?.year || undefined
          );
          if (res.length > 0) {
            await api.scrapeAndSaveMovie(id, res[0].provider_ids);
            successCount++;
          }
        }
      } catch (err) {
        console.error("Failed to scrape item", id, err);
      }
    }

    setIsProcessing(false);
    setStatusMessage(`Batch scrape finished: ${successCount}/${selectedIds.length} items matched.`);
    if (activeSection === "movies") {
      loadMovies();
    } else {
      loadTvShows();
    }
  };

  return (
    <div className="win9x-raised p-2 flex items-center justify-between gap-3 bg-[#e8e8e8] border-t-2 border-[#ffffff] shadow-md flex-shrink-0 select-none text-[0.875rem] z-20">
      <div className="flex items-center gap-2">
        <span className="font-bold text-blue-900 bg-white px-2.5 py-1 win9x-inset">
          Selected: {selectedIds.length} {activeSection === "movies" ? "Movies" : "Shows"}
        </span>
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={handleBatchRename}
          disabled={isProcessing}
          className="win9x-button font-bold text-green-900 text-[0.8125rem]"
          title="Preview and execute batch rename according to Jellyfin standards"
        >
          🏷 Batch Rename ({selectedIds.length})
        </button>

        {activeSection === "movies" && (
          <button
            onClick={handleBatchScrape}
            disabled={isProcessing}
            className="win9x-button font-bold text-blue-900 text-[0.8125rem]"
            title="Automatically scrape highest-confidence metadata for selected items"
          >
            {isProcessing ? "Scraping..." : `🔍 Batch Scrape (${selectedIds.length})`}
          </button>
        )}

        <button
          onClick={clearMultiSelection}
          className="win9x-button text-gray-700 text-[0.8125rem]"
          title="Deselect all (Escape)"
        >
          ✕ Clear Selection
        </button>
      </div>
    </div>
  );
};
