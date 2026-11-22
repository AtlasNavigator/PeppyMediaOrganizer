import React, { useEffect, useState } from "react";
import { useMediaStore } from "../../hooks/useMediaStore";
import { api } from "../../lib/tauri";
import type { EpisodeRemapItem, MovieSearchResult, TvSearchResult } from "../../lib/types";

interface ScrapeModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ScrapeModal: React.FC<ScrapeModalProps> = ({ isOpen, onClose }) => {
  const activeSection = useMediaStore((s) => s.activeSection);
  const movies = useMediaStore((s) => s.movies);
  const selectedMovieId = useMediaStore((s) => s.selectedMovieId);
  const loadMovies = useMediaStore((s) => s.loadMovies);

  const tvShows = useMediaStore((s) => s.tvShows);
  const selectedShowId = useMediaStore((s) => s.selectedShowId);
  const loadTvShows = useMediaStore((s) => s.loadTvShows);

  const movie = movies.find((m) => m.id === selectedMovieId);
  const show = tvShows.find((s) => s.id === selectedShowId);

  const [query, setQuery] = useState("");
  const [year, setYear] = useState<string>("");
  const [isSearching, setIsSearching] = useState(false);
  const [isScraping, setIsScraping] = useState(false);
  const [statusMsg, setStatusMsg] = useState("");

  const [movieResults, setMovieResults] = useState<MovieSearchResult[]>([]);
  const [tvResults, setTvResults] = useState<TvSearchResult[]>([]);
  const [selectedMovieResult, setSelectedMovieResult] = useState<MovieSearchResult | null>(null);
  const [selectedTvResult, setSelectedTvResult] = useState<TvSearchResult | null>(null);

  // TV Episode Ordering
  const [episodeOrder, setEpisodeOrder] = useState<"aired" | "dvd" | "streaming" | "absolute">("aired");
  const [remapPreview, setRemapPreview] = useState<EpisodeRemapItem[]>([]);
  const [isLoadingRemap, setIsLoadingRemap] = useState(false);

  useEffect(() => {
    if (activeSection === "movies" && movie) {
      setQuery(movie.title);
      setYear(movie.year ? movie.year.toString() : "");
      handleSearch(movie.title, movie.year || undefined);
    } else if (activeSection === "tvshows" && show) {
      setQuery(show.title);
      setYear(show.year ? show.year.toString() : "");
      setEpisodeOrder(show.episode_order_mode || "aired");
      handleSearch(show.title, show.year || undefined);
      loadRemapPreview(show.id, show.episode_order_mode || "aired");
    }
  }, [activeSection, movie?.id, show?.id]);

  const loadRemapPreview = async (showId: number, order: string) => {
    try {
      setIsLoadingRemap(true);
      const preview = await api.previewEpisodeRemap(showId, order);
      setRemapPreview(preview);
    } catch (err) {
      console.warn("Remap preview not available yet:", err);
    } finally {
      setIsLoadingRemap(false);
    }
  };

  const handleOrderChange = (newOrder: "aired" | "dvd" | "streaming" | "absolute") => {
    setEpisodeOrder(newOrder);
    if (show) {
      loadRemapPreview(show.id, newOrder);
    }
  };

  const handleSearch = async (searchTitle?: string, searchYear?: number) => {
    const t = searchTitle ?? query;
    if (!t.trim()) return;

    setIsSearching(true);
    setStatusMsg("Searching providers (IMDb, TMDb, TVDb, OMDb)...");

    try {
      const y = searchYear ?? (year ? parseInt(year, 10) : undefined);
      if (activeSection === "movies") {
        const res = await api.searchMetadataMovie(t, y);
        setMovieResults(res);
        if (res.length > 0) {
          setSelectedMovieResult(res[0]);
        }
        setStatusMsg(`Found ${res.length} matches.`);
      } else {
        const res = await api.searchMetadataTv(t, y);
        setTvResults(res);
        if (res.length > 0) {
          setSelectedTvResult(res[0]);
        }
        setStatusMsg(`Found ${res.length} show matches.`);
      }
    } catch (err) {
      setStatusMsg(`Search error: ${err}`);
    } finally {
      setIsSearching(false);
    }
  };

  const handleApplyTv = async () => {
    if (!show) return;
    try {
      setIsScraping(true);
      setStatusMsg("Fetching TV metadata, writing tvshow.nfo and episode NFOs...");
      const providerIds = selectedTvResult?.provider_ids || {
        imdb_id: show.imdb_id || null,
        tmdb_id: show.tmdb_id || null,
        tvdb_id: show.tvdb_id || null,
      };
      await api.scrapeAndSaveTv(show.id, providerIds, episodeOrder);
      await loadTvShows();
      setStatusMsg("✅ TV show and episode NFOs saved successfully!");
      setTimeout(() => onClose(), 800);
    } catch (err) {
      setStatusMsg(`Scrape error: ${err}`);
    } finally {
      setIsScraping(false);
    }
  };

  const handleApplyMovie = async () => {
    if (!movie || !selectedMovieResult) return;
    try {
      setIsScraping(true);
      setStatusMsg("Fetching complete metadata, writing Jellyfin movie.nfo...");
      await api.scrapeAndSaveMovie(movie.id, selectedMovieResult.provider_ids);
      await loadMovies();
      setStatusMsg("✅ Metadata and movie.nfo saved successfully!");
      setTimeout(() => onClose(), 800);
    } catch (err) {
      setStatusMsg(`Scrape error: ${err}`);
    } finally {
      setIsScraping(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
      <div className="w-[920px] max-w-[95vw] max-h-[90vh] win9x-raised flex flex-col bg-[#c0c0c0] shadow-2xl overflow-hidden select-none">
        {/* Title Bar */}
        <div className="win9x-titlebar flex items-center justify-between px-2">
          <span>Metadata Provider Scraper & Episode Ordering</span>
          <button
            onClick={onClose}
            className="win9x-button !min-h-[1.5rem] !px-1.5 text-[0.8125rem] font-bold"
          >
            ✕
          </button>
        </div>

        {/* Content Area */}
        <div className="flex-1 flex flex-col p-2.5 gap-2 overflow-hidden text-[0.875rem]">
          {/* Search Inputs */}
          <div className="win9x-raised p-2 flex items-center gap-2 bg-[#dfdfdf]">
            <span className="font-bold">Title:</span>
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className="win9x-input flex-1 text-[0.875rem]"
              placeholder="Search title..."
            />

            <span className="font-bold">Year:</span>
            <input
              type="text"
              value={year}
              onChange={(e) => setYear(e.target.value)}
              className="win9x-input w-20 text-center text-[0.875rem]"
              placeholder="YYYY"
            />

            <button
              onClick={() => handleSearch()}
              disabled={isSearching}
              className="win9x-button font-bold text-blue-900 text-[0.8125rem]"
            >
              {isSearching ? "Searching..." : "🔍 Search Providers"}
            </button>
          </div>

          {/* Body Section */}
          <div className="flex-1 flex gap-2 min-h-0 overflow-hidden">
            {/* Search Results Table */}
            <div className="flex-1 flex flex-col win9x-inset overflow-hidden bg-white">
              <div className="win9x-th flex-shrink-0 bg-[#c0c0c0]">
                Provider Search Matches
              </div>
              <div className="flex-1 overflow-y-auto">
                {activeSection === "movies" ? (
                  movieResults.length === 0 ? (
                    <div className="p-4 text-center text-gray-500">
                      {isSearching ? "Querying metadata providers..." : "No matches found."}
                    </div>
                  ) : (
                    movieResults.map((m, idx) => {
                      const isSelected = selectedMovieResult?.provider_ids.imdb_id === m.provider_ids.imdb_id;
                      return (
                        <div
                          key={idx}
                          onClick={() => setSelectedMovieResult(m)}
                          className={`win9x-tr flex items-center px-2 py-1 border-b border-[#f0f0f0] gap-2 ${
                            isSelected ? "selected" : ""
                          }`}
                        >
                          <div className="flex-1 font-bold truncate">
                            {m.title} {m.year && <span className="font-normal">({m.year})</span>}
                          </div>
                          <div className="font-mono text-[0.8125rem] space-x-1">
                            {m.provider_ids.imdb_id && (
                              <span className="text-green-800">[{m.provider_ids.imdb_id}]</span>
                            )}
                            {m.provider_ids.tmdb_id && (
                              <span className="text-blue-800">[TMDb:{m.provider_ids.tmdb_id}]</span>
                            )}
                          </div>
                          {m.rating && (
                            <div className="text-[0.8125rem] font-bold text-yellow-800">
                              ★ {m.rating.toFixed(1)}
                            </div>
                          )}
                        </div>
                      );
                    })
                  )
                ) : (
                  tvResults.length === 0 ? (
                    <div className="p-4 text-center text-gray-500">
                      {isSearching ? "Querying metadata providers..." : "No TV show matches found."}
                    </div>
                  ) : (
                    tvResults.map((s, idx) => {
                      const isSelected =
                        (selectedTvResult?.provider_ids.tvdb_id && selectedTvResult.provider_ids.tvdb_id === s.provider_ids.tvdb_id) ||
                        (selectedTvResult?.provider_ids.imdb_id && selectedTvResult.provider_ids.imdb_id === s.provider_ids.imdb_id) ||
                        (selectedTvResult?.provider_ids.tmdb_id && selectedTvResult.provider_ids.tmdb_id === s.provider_ids.tmdb_id);
                      return (
                        <div
                          key={idx}
                          onClick={() => setSelectedTvResult(s)}
                          className={`win9x-tr flex items-center px-2 py-1.5 border-b border-[#f0f0f0] gap-2 cursor-pointer ${
                            isSelected ? "selected" : ""
                          }`}
                        >
                          <div className="flex-1 font-bold truncate">
                            {s.title} {s.year && <span className="font-normal">({s.year})</span>}
                          </div>
                          <div className="font-mono text-[0.8125rem] space-x-1">
                            {s.provider_ids.tvdb_id && (
                              <span className="text-blue-800">[TVDb:{s.provider_ids.tvdb_id}]</span>
                            )}
                            {s.provider_ids.tmdb_id && (
                              <span className="text-blue-800">[TMDb:{s.provider_ids.tmdb_id}]</span>
                            )}
                            {s.provider_ids.imdb_id && (
                              <span className="text-green-800">[{s.provider_ids.imdb_id}]</span>
                            )}
                          </div>
                          {s.rating && (
                            <div className="text-[0.8125rem] font-bold text-yellow-800">
                              ★ {s.rating.toFixed(1)}
                            </div>
                          )}
                        </div>
                      );
                    })
                  )
                )}
              </div>
            </div>

            {/* Right Side: TV Order Preview or Movie Poster Preview */}
            <div className="w-80 flex flex-col gap-2 flex-shrink-0 overflow-hidden">
              {activeSection === "tvshows" ? (
                <div className="flex-1 flex flex-col win9x-raised p-2 overflow-hidden bg-[#dfdfdf]">
                  <div className="font-bold border-b border-[#808080] pb-1 mb-1 flex items-center justify-between">
                    <span>Episode Ordering Mode</span>
                    <select
                      value={episodeOrder}
                      onChange={(e) => handleOrderChange(e.target.value as any)}
                      className="win9x-input font-bold"
                    >
                      <option value="aired">Aired (Broadcast)</option>
                      <option value="dvd">DVD Order</option>
                      <option value="streaming">Streaming / Digital</option>
                      <option value="absolute">Absolute (Anime 1..N)</option>
                    </select>
                  </div>

                  <div className="text-[10px] text-gray-600 mb-1">
                    Preview file remapping based on {episodeOrder.toUpperCase()} order:
                  </div>

                  <div className="flex-1 win9x-inset overflow-y-auto bg-white p-1">
                    {isLoadingRemap ? (
                      <div className="p-3 text-center text-gray-500">Calculating order mapping...</div>
                    ) : remapPreview.length === 0 ? (
                      <div className="p-3 text-center text-gray-500">No episode mapping found.</div>
                    ) : (
                      remapPreview.map((item, idx) => (
                        <div
                          key={idx}
                          className={`p-1 border-b border-[#f0f0f0] text-[10px] ${
                            item.has_changed ? "bg-amber-50" : ""
                          }`}
                        >
                          <div className="flex justify-between font-mono">
                            <span className="text-gray-500">
                              S{item.current_season.toString().padStart(2, "0")}E{item.current_episode.toString().padStart(2, "0")}
                            </span>
                            <span>→</span>
                            <span className="font-bold text-blue-900">
                              S{item.target_season.toString().padStart(2, "0")}E{item.target_episode.toString().padStart(2, "0")}
                            </span>
                          </div>
                          <div className="truncate text-[9px] text-gray-700">
                            {item.target_title || item.current_title || "Episode"}
                          </div>
                        </div>
                      ))
                    )}
                  </div>
                </div>
              ) : (
                <div className="flex-1 flex flex-col win9x-panel-inset p-2 overflow-y-auto">
                  {selectedMovieResult?.poster_url ? (
                    <img
                      src={selectedMovieResult.poster_url}
                      alt="Poster Preview"
                      className="max-h-56 mx-auto win9x-raised mb-2 object-contain"
                    />
                  ) : (
                    <div className="h-44 win9x-inset flex items-center justify-center text-gray-400 mb-2">
                      No Poster URL
                    </div>
                  )}

                  <div className="font-bold text-[0.9375rem] text-black">
                    {selectedMovieResult?.title || "No selection"}
                  </div>
                  <div className="text-gray-600 text-[0.8125rem] mt-0.5">
                    {selectedMovieResult?.year ? `(${selectedMovieResult.year})` : ""}
                  </div>

                  {selectedMovieResult?.overview && (
                    <p className="text-[0.8125rem] text-gray-800 mt-2 leading-relaxed max-h-28 overflow-y-auto">
                      {selectedMovieResult.overview}
                    </p>
                  )}
                </div>
              )}
            </div>
          </div>

          {/* Footer Controls */}
          <div className="win9x-raised p-2 flex items-center justify-between bg-[#c0c0c0]">
            <div className="text-blue-900 font-bold truncate flex-1 pr-2 text-[0.875rem]">
              {statusMsg}
            </div>

            <div className="flex items-center gap-2">
              <button onClick={onClose} className="win9x-button text-[0.8125rem]">
                Cancel
              </button>
              {activeSection === "movies" ? (
                <button
                  onClick={handleApplyMovie}
                  disabled={isScraping || !selectedMovieResult}
                  className="win9x-button font-bold text-green-900 text-[0.8125rem] !px-4"
                >
                  {isScraping ? "Applying..." : "✓ Apply & Generate Jellyfin NFO"}
                </button>
              ) : (
                <button
                  onClick={handleApplyTv}
                  disabled={isScraping || (!selectedTvResult && !show?.tvdb_id && !show?.imdb_id && !show?.tmdb_id)}
                  className="win9x-button font-bold text-green-900 text-[0.8125rem] !px-4"
                >
                  {isScraping ? "Applying..." : "✓ Apply Metadata & Order NFOs"}
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
