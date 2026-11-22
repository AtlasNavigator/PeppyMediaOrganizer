import React, { useEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useMediaStore } from "../../hooks/useMediaStore";

const CARD_WIDTH = 180;
const CARD_HEIGHT = 290;
const GAP = 12;

export const PosterGrid: React.FC = () => {
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

  const containerRef = useRef<HTMLDivElement>(null);
  const [columns, setColumns] = useState(6);

  // Resize observer to calculate dynamic column count
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const updateColumns = () => {
      const width = el.clientWidth - 16; // padding
      const cols = Math.max(1, Math.floor(width / (CARD_WIDTH + GAP)));
      setColumns(cols);
    };

    updateColumns();
    const observer = new ResizeObserver(updateColumns);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Filtered lists
  const filteredMovies = useMemo(() => {
    let list = movies;
    if (searchQuery) {
      list = list.filter(
        (m) =>
          m.title.toLowerCase().includes(searchQuery) ||
          (m.year && m.year.toString().includes(searchQuery)) ||
          (m.imdb_id && m.imdb_id.toLowerCase().includes(searchQuery))
      );
    }
    if (statusFilter === "matched") list = list.filter((m) => m.imdb_id || m.tmdb_id);
    else if (statusFilter === "unmatched") list = list.filter((m) => !m.imdb_id && !m.tmdb_id);
    else if (statusFilter === "missing_nfo") list = list.filter((m) => m.has_nfo === 0);
    else if (statusFilter === "missing_poster") list = list.filter((m) => m.has_poster === 0);

    return list;
  }, [movies, searchQuery, statusFilter]);

  const filteredShows = useMemo(() => {
    let list = tvShows;
    if (searchQuery) {
      list = list.filter(
        (s) =>
          s.title.toLowerCase().includes(searchQuery) ||
          (s.year && s.year.toString().includes(searchQuery)) ||
          (s.tvdb_id && s.tvdb_id.toString().includes(searchQuery))
      );
    }
    if (statusFilter === "matched") list = list.filter((s) => s.tvdb_id || s.tmdb_id || s.imdb_id);
    else if (statusFilter === "unmatched") list = list.filter((s) => !s.tvdb_id && !s.tmdb_id && !s.imdb_id);
    else if (statusFilter === "missing_nfo") list = list.filter((s) => s.has_nfo === 0);
    else if (statusFilter === "missing_poster") list = list.filter((s) => s.has_poster === 0);

    return list;
  }, [tvShows, searchQuery, statusFilter]);

  const items = activeSection === "movies" ? filteredMovies : filteredShows;
  const allOrderedIds = useMemo(() => items.map((i) => i.id), [items]);
  const rowCount = Math.ceil(items.length / columns);

  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => containerRef.current,
    estimateSize: () => CARD_HEIGHT + GAP,
    overscan: 3,
  });

  const handleCardClick = (e: React.MouseEvent, id: number) => {
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

  // 2D Keyboard Navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (items.length === 0) return;

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

    const currentId = activeSection === "movies" ? selectedMovieId : selectedShowId;
    const currentIndex = items.findIndex((i) => i.id === currentId);
    let nextIndex = currentIndex;

    if (e.key === "ArrowRight") {
      e.preventDefault();
      nextIndex = Math.min(currentIndex + 1, items.length - 1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      nextIndex = Math.max(currentIndex - 1, 0);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      nextIndex = Math.min(currentIndex + columns, items.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      nextIndex = Math.max(currentIndex - columns, 0);
    }

    if (nextIndex !== currentIndex && nextIndex >= 0) {
      const targetItem = items[nextIndex];
      if (activeSection === "movies") {
        selectMovie(targetItem.id);
      } else {
        selectShow(targetItem.id);
      }
      const targetRow = Math.floor(nextIndex / columns);
      rowVirtualizer.scrollToIndex(targetRow);
    }
  };

  return (
    <div
      ref={containerRef}
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="flex-1 win9x-inset p-2 overflow-y-auto outline-none select-none bg-[#b0b0b0]"
    >
      {items.length === 0 ? (
        <div className="h-full flex items-center justify-center text-gray-700 text-xs font-bold">
          No media matching current filter or search criteria.
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
            const rowStartIndex = virtualRow.index * columns;
            const rowItems = items.slice(rowStartIndex, rowStartIndex + columns);

            return (
              <div
                key={virtualRow.index}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  height: `${CARD_HEIGHT}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                  display: "flex",
                  gap: `${GAP}px`,
                }}
              >
                {rowItems.map((item) => {
                  const isMovie = activeSection === "movies";
                  const isSelected = isMovie
                    ? selectedMovieIds.has(item.id)
                    : selectedShowIds.has(item.id);
                  const isFocused = isMovie
                    ? item.id === selectedMovieId
                    : item.id === selectedShowId;

                  const hasPoster = item.has_poster === 1;
                  const posterUrl = isMovie
                    ? (item as any).poster_url
                    : (item as any).poster_url;
                  const res = (item as any).resolution;
                  const isMatched = item.scan_status === "matched";

                  return (
                    <div
                      key={item.id}
                      onClick={(e) => handleCardClick(e, item.id)}
                      style={{ width: `${CARD_WIDTH}px`, height: `${CARD_HEIGHT}px` }}
                      className={`flex flex-col p-1.5 cursor-pointer transition-none text-left relative ${
                        isSelected
                          ? "win9x-inset bg-[#000080] text-white"
                          : isFocused
                          ? "win9x-raised bg-[#d0d0e0] text-black"
                          : "win9x-raised bg-[#c0c0c0] text-black hover:bg-[#dfdfdf]"
                      }`}
                      title={`${item.title} (${item.year || "?"})\nStatus: ${item.scan_status}`}
                    >
                      {/* Poster Area */}
                      <div className="w-full flex-1 win9x-inset bg-[#202020] relative overflow-hidden flex items-center justify-center">
                        {posterUrl ? (
                          <img
                            src={posterUrl}
                            alt={item.title}
                            className="w-full h-full object-cover"
                            loading="lazy"
                          />
                        ) : hasPoster ? (
                          <div className="w-full h-full bg-[#183048] flex flex-col items-center justify-center text-white text-[0.8125rem] p-1.5 text-center">
                            <span className="text-2xl">🎬</span>
                            <span className="font-bold truncate mt-1">Local Poster</span>
                          </div>
                        ) : (
                          <div className="flex flex-col items-center justify-center text-gray-400 text-[0.8125rem] text-center p-2">
                            <span className="text-3xl mb-1">📼</span>
                            <span>No Poster</span>
                          </div>
                        )}

                        {/* Top Badges */}
                        <div className="absolute top-1 left-1 flex flex-col gap-1">
                          {res && (
                            <span className="px-1.5 py-0.5 bg-black/80 text-white font-mono font-bold text-[0.75rem] rounded border border-gray-400">
                              {res}
                            </span>
                          )}
                          {!isMatched && (
                            <span className="px-1.5 py-0.5 bg-yellow-400 text-black font-bold text-[0.75rem] rounded border border-yellow-600">
                              ⚠ Unmatched
                            </span>
                          )}
                        </div>

                        {/* Multi-Select Checkbox */}
                        <div
                          className="absolute top-1 right-1"
                          onClick={(e) => {
                            e.stopPropagation();
                            toggleItemSelection(item.id);
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={isSelected}
                            readOnly
                            className="cursor-pointer w-4 h-4"
                          />
                        </div>
                      </div>

                      {/* Card Caption */}
                      <div className="h-12 mt-1.5 flex flex-col justify-center px-1 overflow-hidden">
                        <div className="font-bold text-[0.875rem] leading-tight truncate">
                          {item.title}
                        </div>
                        <div className="flex justify-between items-center text-[0.8125rem] opacity-80 mt-0.5 font-mono">
                          <span>{item.year || "—"}</span>
                          {item.has_nfo === 1 && (
                            <span className="text-[0.75rem] px-1 bg-green-200 text-green-900 border border-green-400">
                              NFO
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
