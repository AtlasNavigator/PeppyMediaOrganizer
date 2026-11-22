import React, { useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useMediaStore } from "../../hooks/useMediaStore";

type SortKey = "title" | "year" | "resolution" | "status";
type SortOrder = "asc" | "desc";

export const MediaTable: React.FC = () => {
  const activeSection = useMediaStore((s) => s.activeSection);
  const movies = useMediaStore((s) => s.movies);
  const selectedMovieId = useMediaStore((s) => s.selectedMovieId);
  const selectMovie = useMediaStore((s) => s.selectMovie);

  const tvShows = useMediaStore((s) => s.tvShows);
  const selectedShowId = useMediaStore((s) => s.selectedShowId);
  const selectShow = useMediaStore((s) => s.selectShow);

  const selectedMovieIds = useMediaStore((s) => s.selectedMovieIds);
  const selectedShowIds = useMediaStore((s) => s.selectedShowIds);
  const toggleItemSelection = useMediaStore((s) => s.toggleItemSelection);
  const selectItemRange = useMediaStore((s) => s.selectItemRange);
  const selectAllFiltered = useMediaStore((s) => s.selectAllFiltered);
  const clearMultiSelection = useMediaStore((s) => s.clearMultiSelection);

  const searchQuery = useMediaStore((s) => s.searchQuery).toLowerCase().trim();
  const statusFilter = useMediaStore((s) => s.statusFilter);

  const [sortKey, setSortKey] = useState<SortKey>("title");
  const [sortOrder, setSortOrder] = useState<SortOrder>("asc");

  const parentRef = useRef<HTMLDivElement>(null);

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortOrder(sortOrder === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortOrder("asc");
    }
  };

  // Filter and sort items
  const filteredMovies = useMemo(() => {
    let list = movies;

    if (searchQuery) {
      list = list.filter(
        (m) =>
          m.title.toLowerCase().includes(searchQuery) ||
          (m.year && m.year.toString().includes(searchQuery)) ||
          (m.imdb_id && m.imdb_id.toLowerCase().includes(searchQuery)) ||
          m.folder_path.toLowerCase().includes(searchQuery)
      );
    }

    if (statusFilter === "matched") {
      list = list.filter((m) => m.imdb_id || m.tmdb_id);
    } else if (statusFilter === "unmatched") {
      list = list.filter((m) => !m.imdb_id && !m.tmdb_id);
    } else if (statusFilter === "missing_nfo") {
      list = list.filter((m) => m.has_nfo === 0);
    } else if (statusFilter === "missing_poster") {
      list = list.filter((m) => m.has_poster === 0);
    }

    return [...list].sort((a, b) => {
      let cmp = 0;
      if (sortKey === "title") {
        cmp = a.title.localeCompare(b.title);
      } else if (sortKey === "year") {
        cmp = (a.year || 0) - (b.year || 0);
      } else if (sortKey === "resolution") {
        cmp = (a.resolution || "").localeCompare(b.resolution || "");
      } else if (sortKey === "status") {
        cmp = a.scan_status.localeCompare(b.scan_status);
      }
      return sortOrder === "asc" ? cmp : -cmp;
    });
  }, [movies, searchQuery, statusFilter, sortKey, sortOrder]);

  const filteredShows = useMemo(() => {
    let list = tvShows;

    if (searchQuery) {
      list = list.filter(
        (s) =>
          s.title.toLowerCase().includes(searchQuery) ||
          (s.year && s.year.toString().includes(searchQuery)) ||
          (s.tvdb_id && s.tvdb_id.toString().includes(searchQuery)) ||
          s.folder_path.toLowerCase().includes(searchQuery)
      );
    }

    if (statusFilter === "matched") {
      list = list.filter((s) => s.tvdb_id || s.tmdb_id || s.imdb_id);
    } else if (statusFilter === "unmatched") {
      list = list.filter((s) => !s.tvdb_id && !s.tmdb_id && !s.imdb_id);
    } else if (statusFilter === "missing_nfo") {
      list = list.filter((s) => s.has_nfo === 0);
    } else if (statusFilter === "missing_poster") {
      list = list.filter((s) => s.has_poster === 0);
    }

    return [...list].sort((a, b) => {
      let cmp = 0;
      if (sortKey === "title") {
        cmp = a.title.localeCompare(b.title);
      } else if (sortKey === "year") {
        cmp = (a.year || 0) - (b.year || 0);
      } else if (sortKey === "status") {
        cmp = a.scan_status.localeCompare(b.scan_status);
      }
      return sortOrder === "asc" ? cmp : -cmp;
    });
  }, [tvShows, searchQuery, statusFilter, sortKey, sortOrder]);

  const itemCount = activeSection === "movies" ? filteredMovies.length : filteredShows.length;
  const allOrderedIds = activeSection === "movies" ? filteredMovies.map((m) => m.id) : filteredShows.map((s) => s.id);
  const selectedSet = activeSection === "movies" ? selectedMovieIds : selectedShowIds;
  const isAllSelected = allOrderedIds.length > 0 && allOrderedIds.every((id) => selectedSet.has(id));

  const handleRowClick = (e: React.MouseEvent, id: number) => {
    if (e.shiftKey) {
      selectItemRange(id, allOrderedIds);
    } else if (e.ctrlKey || e.metaKey) {
      toggleItemSelection(id);
    } else {
      if (activeSection === "movies") {
        selectMovie(id);
      } else {
        selectShow(id);
      }
    }
  };

  const handleToggleAll = () => {
    if (isAllSelected) {
      clearMultiSelection();
    } else {
      selectAllFiltered(allOrderedIds);
    }
  };

  const rowVirtualizer = useVirtualizer({
    count: itemCount,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 30,
    overscan: 10,
  });

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (itemCount === 0) return;

    if (e.ctrlKey && (e.key === "a" || e.key === "A")) {
      e.preventDefault();
      selectAllFiltered(allOrderedIds);
      return;
    }

    if (e.key === "Escape") {
      e.preventDefault();
      clearMultiSelection();
      return;
    }

    if (activeSection === "movies") {
      const currentIndex = filteredMovies.findIndex((m) => m.id === selectedMovieId);
      if (e.key === "ArrowDown") {
        e.preventDefault();
        const next = Math.min(currentIndex + 1, filteredMovies.length - 1);
        selectMovie(filteredMovies[next].id);
        rowVirtualizer.scrollToIndex(next);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        const prev = Math.max(currentIndex - 1, 0);
        selectMovie(filteredMovies[prev].id);
        rowVirtualizer.scrollToIndex(prev);
      }
    } else {
      const currentIndex = filteredShows.findIndex((s) => s.id === selectedShowId);
      if (e.key === "ArrowDown") {
        e.preventDefault();
        const next = Math.min(currentIndex + 1, filteredShows.length - 1);
        selectShow(filteredShows[next].id);
        rowVirtualizer.scrollToIndex(next);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        const prev = Math.max(currentIndex - 1, 0);
        selectShow(filteredShows[prev].id);
        rowVirtualizer.scrollToIndex(prev);
      }
    }
  };

  const getSortIndicator = (key: SortKey) => {
    if (sortKey !== key) return "";
    return sortOrder === "asc" ? " ▲" : " ▼";
  };

  return (
    <div
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="flex-1 flex flex-col win9x-inset overflow-hidden outline-none select-none"
    >
      {/* Table Header */}
      <div className="flex bg-[#c0c0c0] border-b border-[#808080] flex-shrink-0 z-10 text-[0.875rem]">
        <div className="win9x-th w-8 text-center flex items-center justify-center">
          <input
            type="checkbox"
            checked={isAllSelected}
            onChange={handleToggleAll}
            className="cursor-pointer"
            title="Select / Deselect All (Ctrl+A)"
          />
        </div>
        {activeSection === "movies" ? (
          <>
            <div
              onClick={() => handleSort("title")}
              className="win9x-th flex-1 min-w-[200px]"
            >
              Title{getSortIndicator("title")}
            </div>
            <div
              onClick={() => handleSort("year")}
              className="win9x-th w-20 text-center"
            >
              Year{getSortIndicator("year")}
            </div>
            <div
              onClick={() => handleSort("resolution")}
              className="win9x-th w-24 text-center"
            >
              Res{getSortIndicator("resolution")}
            </div>
            <div className="win9x-th w-28 text-center">Provider ID</div>
            <div className="win9x-th w-28 text-center">Sidecars</div>
            <div
              onClick={() => handleSort("status")}
              className="win9x-th w-24 text-center"
            >
              Status{getSortIndicator("status")}
            </div>
          </>
        ) : (
          <>
            <div
              onClick={() => handleSort("title")}
              className="win9x-th flex-1 min-w-[200px]"
            >
              Series Title{getSortIndicator("title")}
            </div>
            <div
              onClick={() => handleSort("year")}
              className="win9x-th w-20 text-center"
            >
              Year{getSortIndicator("year")}
            </div>
            <div className="win9x-th w-32 text-center">Provider ID</div>
            <div className="win9x-th w-28 text-center">Sidecars</div>
            <div
              onClick={() => handleSort("status")}
              className="win9x-th w-24 text-center"
            >
              Status{getSortIndicator("status")}
            </div>
          </>
        )}
      </div>

      {/* Table Body */}
      <div ref={parentRef} className="flex-1 overflow-auto bg-white">
        {itemCount === 0 ? (
          <div className="h-full flex items-center justify-center text-[#808080] text-[0.9375rem]">
            {movies.length === 0 && tvShows.length === 0
              ? "Library is empty. Click 'Scan Library' or configure library roots in Settings."
              : "No items match current filter/search criteria."}
          </div>
        ) : (
          <div
            style={{
              height: `${rowVirtualizer.getTotalSize()}px`,
              width: "100%",
              position: "relative",
            }}
          >
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const index = virtualRow.index;

              if (activeSection === "movies") {
                const movie = filteredMovies[index];
                const isSelected = selectedMovieIds.has(movie.id);
                const isFocused = movie.id === selectedMovieId;
                const hasId = movie.imdb_id || movie.tmdb_id;

                return (
                  <div
                    key={movie.id}
                    onClick={(e) => handleRowClick(e, movie.id)}
                    style={{
                      position: "absolute",
                      top: 0,
                      left: 0,
                      width: "100%",
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                    className={`win9x-tr flex items-center px-1.5 border-b border-[#f0f0f0] text-[0.875rem] ${
                      isSelected
                        ? "selected font-semibold"
                        : isFocused
                        ? "bg-[#e8f0fe]"
                        : index % 2 === 1
                        ? "bg-[#f9f9f9]"
                        : "bg-white"
                    }`}
                  >
                    <div className="w-8 text-center flex items-center justify-center">
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => toggleItemSelection(movie.id)}
                        onClick={(e) => e.stopPropagation()}
                        className="cursor-pointer"
                      />
                    </div>
                    <div className="flex-1 truncate font-medium pr-2">
                      {movie.title}
                      {movie.version_label && (
                        <span className="ml-1.5 px-1 py-0.2 bg-[#e0e0e0] text-[#404040] text-[0.75rem] border border-[#a0a0a0]">
                          {movie.version_label}
                        </span>
                      )}
                    </div>
                    <div className="w-20 text-center text-[#404040]">
                      {movie.year || "-"}
                    </div>
                    <div className="w-24 text-center font-mono text-[0.8125rem]">
                      {movie.resolution || "-"}
                    </div>
                    <div className="w-28 text-center font-mono text-[0.8125rem]">
                      {hasId ? (
                        <span className="text-green-700 font-bold">✓ {movie.imdb_id ? "IMDb" : "TMDb"}</span>
                      ) : (
                        <span className="text-red-600">✗ Missing</span>
                      )}
                    </div>
                    <div className="w-28 text-center text-[0.8125rem] space-x-1.5">
                      <span title={movie.has_nfo ? "NFO Present" : "Missing NFO"} className={movie.has_nfo ? "text-green-700" : "text-gray-300"}>
                        NFO
                      </span>
                      <span title={movie.has_poster ? "Poster Present" : "Missing Poster"} className={movie.has_poster ? "text-blue-700" : "text-gray-300"}>
                        ART
                      </span>
                      {movie.subtitle_count > 0 && (
                        <span className="text-purple-700 font-bold">
                          [{movie.subtitle_count}]
                        </span>
                      )}
                    </div>
                    <div className="w-24 text-center text-[0.8125rem]">
                      <span
                        className={`px-1.5 py-0.5 font-semibold ${
                          movie.scan_status === "matched"
                            ? "bg-green-100 text-green-800"
                            : "bg-yellow-100 text-yellow-800"
                        }`}
                      >
                        {movie.scan_status}
                      </span>
                    </div>
                  </div>
                );
              } else {
                const show = filteredShows[index];
                const isSelected = selectedShowIds.has(show.id);
                const isFocused = show.id === selectedShowId;
                const hasId = show.tvdb_id || show.tmdb_id || show.imdb_id;

                return (
                  <div
                    key={show.id}
                    onClick={(e) => handleRowClick(e, show.id)}
                    style={{
                      position: "absolute",
                      top: 0,
                      left: 0,
                      width: "100%",
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                    className={`win9x-tr flex items-center px-1.5 border-b border-[#f0f0f0] text-[0.875rem] ${
                      isSelected
                        ? "selected font-semibold"
                        : isFocused
                        ? "bg-[#e8f0fe]"
                        : index % 2 === 1
                        ? "bg-[#f9f9f9]"
                        : "bg-white"
                    }`}
                  >
                    <div className="w-8 text-center flex items-center justify-center">
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => toggleItemSelection(show.id)}
                        onClick={(e) => e.stopPropagation()}
                        className="cursor-pointer"
                      />
                    </div>
                    <div className="flex-1 truncate font-medium pr-2">
                      {show.title}
                    </div>
                    <div className="w-20 text-center text-[#404040]">
                      {show.year || "-"}
                    </div>
                    <div className="w-32 text-center font-mono text-[0.8125rem]">
                      {hasId ? (
                        <span className="text-green-700 font-bold">
                          ✓ {show.tvdb_id ? `TVDB:${show.tvdb_id}` : show.imdb_id ? "IMDb" : "TMDb"}
                        </span>
                      ) : (
                        <span className="text-red-600">✗ Missing</span>
                      )}
                    </div>
                    <div className="w-28 text-center text-[0.8125rem] space-x-1.5">
                      <span title={show.has_nfo ? "NFO Present" : "Missing NFO"} className={show.has_nfo ? "text-green-700" : "text-gray-300"}>
                        NFO
                      </span>
                      <span title={show.has_poster ? "Poster Present" : "Missing Poster"} className={show.has_poster ? "text-blue-700" : "text-gray-300"}>
                        ART
                      </span>
                    </div>
                    <div className="w-24 text-center text-[0.8125rem]">
                      <span
                        className={`px-1.5 py-0.5 font-semibold ${
                          show.scan_status === "matched"
                            ? "bg-green-100 text-green-800"
                            : "bg-yellow-100 text-yellow-800"
                        }`}
                      >
                        {show.scan_status}
                      </span>
                    </div>
                  </div>
                );
              }
            })}
          </div>
        )}
      </div>
    </div>
  );
};
