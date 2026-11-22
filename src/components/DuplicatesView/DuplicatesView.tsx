import React, { useState } from "react";
import { useMediaStore } from "../../hooks/useMediaStore";
import type { DuplicateMovieItem, DuplicateEpisodeItem } from "../../lib/types";

export const DuplicatesView: React.FC = () => {
  const movieDuplicates = useMediaStore((s) => s.movieDuplicates);
  const episodeDuplicates = useMediaStore((s) => s.episodeDuplicates);
  const isLoading = useMediaStore((s) => s.isLoadingDuplicates);
  const loadDuplicates = useMediaStore((s) => s.loadDuplicates);
  const startRenamePlan = useMediaStore((s) => s.startRenamePlan);
  const resolveDuplicatesKeepSingle = useMediaStore((s) => s.resolveDuplicatesKeepSingle);
  const selectMovie = useMediaStore((s) => s.selectMovie);
  const setActiveSection = useMediaStore((s) => s.setActiveSection);

  const [activeTab, setActiveTab] = useState<"movies" | "episodes">("movies");
  const [copiedPath, setCopiedPath] = useState<string | null>(null);

  const formatFileSize = (bytes?: number | null) => {
    if (!bytes || bytes === 0) return "0 B (Mock / Stub)";
    const mb = bytes / (1024 * 1024);
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(2)} GB`;
    }
    return `${mb.toFixed(1)} MB`;
  };

  const handleCopyPath = (path: string) => {
    navigator.clipboard.writeText(path);
    setCopiedPath(path);
    setTimeout(() => setCopiedPath(null), 2000);
  };

  const handleInspectMovie = (movieId: number) => {
    selectMovie(movieId);
    setActiveSection("movies");
  };

  const handleKeepOnlyMovie = (keptItem: DuplicateMovieItem, allItems: DuplicateMovieItem[]) => {
    const obsoletePaths = allItems
      .filter((i) => i.id !== keptItem.id)
      .map((i) => i.file_path);
    resolveDuplicatesKeepSingle(keptItem.file_path, obsoletePaths, "movie");
  };

  const handleKeepOnlyEpisode = (keptItem: DuplicateEpisodeItem, allItems: DuplicateEpisodeItem[]) => {
    const obsoletePaths = allItems
      .filter((i) => i.id !== keptItem.id)
      .map((i) => i.file_path);
    resolveDuplicatesKeepSingle(keptItem.file_path, obsoletePaths, "episode");
  };

  return (
    <div className="flex-1 flex flex-col p-2 win9x-raised overflow-hidden select-none">
      {/* Title & Controls Bar */}
      <div className="win9x-titlebar mb-2 px-2 py-1 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="font-bold text-[0.9375rem]">👥 Duplicate Media Finder</span>
          <span className="text-[0.8125rem] opacity-80">
            (Identify identical content across library folders)
          </span>
        </div>
        <button
          onClick={() => loadDuplicates()}
          disabled={isLoading}
          className="win9x-button text-[0.8125rem] font-bold !min-h-[1.65rem] !py-0.5 !px-2.5"
        >
          {isLoading ? "⏳ Scanning..." : "⟳ Rescan Duplicates"}
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-[#808080] mb-2 px-1">
        <button
          onClick={() => setActiveTab("movies")}
          className={`win9x-tab ${activeTab === "movies" ? "active" : ""}`}
        >
          🎬 Movies ({movieDuplicates.length} groups)
        </button>
        <button
          onClick={() => setActiveTab("episodes")}
          className={`win9x-tab ${activeTab === "episodes" ? "active" : ""}`}
        >
          📺 TV Episodes ({episodeDuplicates.length} groups)
        </button>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 win9x-panel-inset p-2 overflow-y-auto space-y-3">
        {isLoading && (
          <div className="p-8 text-center text-xs text-gray-600">
            Analyzing database for matching IMDb/TMDb IDs and normalized titles...
          </div>
        )}

        {!isLoading && activeTab === "movies" && (
          <>
            {movieDuplicates.length === 0 ? (
              <div className="win9x-raised p-6 text-center bg-[#c0c0c0]">
                <div className="text-xl mb-1">🎉</div>
                <div className="font-bold text-sm text-black mb-1">No Movie Duplicates Found!</div>
                <div className="text-xs text-gray-700">
                  All movies in your scanned libraries have unique provider IDs and titles.
                </div>
              </div>
            ) : (
              movieDuplicates.map((group, gIdx) => {
                const sampleItem = group.items[0];
                return (
                  <div
                    key={`${group.match_key}-${gIdx}`}
                    className="win9x-raised p-2 bg-[#d4d0c8] space-y-1.5"
                  >
                    {/* Group Header */}
                    <div className="flex items-center justify-between border-b border-[#808080] pb-1">
                      <div className="flex items-center gap-2">
                        <span className="font-bold text-xs text-black">
                          {sampleItem?.title || group.match_key} {sampleItem?.year ? `(${sampleItem.year})` : ""}
                        </span>
                        <span className="bg-[#000080] text-white text-[10px] font-mono px-1.5 py-0.5">
                          {group.reason}
                        </span>
                      </div>
                      <span className="text-[10px] text-gray-700 font-bold">
                        {group.items.length} copies found
                      </span>
                    </div>

                  {/* Candidate Items Table */}
                  <div className="win9x-inset bg-white p-1 overflow-x-auto">
                    <table className="w-full text-[0.875rem] border-collapse">
                      <thead>
                        <tr className="bg-[#c0c0c0] border-b border-[#808080] text-left">
                          <th className="p-1.5 w-24">Quality</th>
                          <th className="p-1.5">File Path</th>
                          <th className="p-1.5 w-28">Resolution</th>
                          <th className="p-1.5 w-28">Size</th>
                          <th className="p-1.5 w-72 text-right">Actions</th>
                        </tr>
                      </thead>
                      <tbody>
                        {group.items.map((item: DuplicateMovieItem, idx: number) => {
                          const isPrimary = idx === 0;
                          return (
                            <tr
                              key={item.id}
                              className={`border-b border-[#e8e8e8] ${
                                isPrimary ? "bg-[#f5f9ff]" : "hover:bg-[#f9f9f9]"
                              }`}
                            >
                              <td className="p-1.5 font-bold">
                                {isPrimary ? (
                                  <span className="text-blue-800 text-[0.8125rem] flex items-center gap-0.5">
                                    ★ Highest
                                  </span>
                                ) : (
                                  <span className="text-gray-500 text-[0.8125rem]">Version #{idx + 1}</span>
                                )}
                              </td>
                              <td className="p-1.5 font-mono text-[0.8125rem] text-gray-800 break-all">
                                {item.file_path}
                              </td>
                              <td className="p-1.5 font-mono text-[0.8125rem]">
                                {item.resolution || "Unknown"}
                                {item.version_label ? ` (${item.version_label})` : ""}
                              </td>
                              <td className="p-1.5 font-mono text-[0.8125rem]">
                                {formatFileSize(item.file_size)}
                              </td>
                              <td className="p-1.5 text-right space-x-1.5 whitespace-nowrap">
                                <button
                                  onClick={() => handleCopyPath(item.file_path)}
                                  className="win9x-button text-[0.8125rem] !py-0.2 !px-2"
                                  title="Copy file path to clipboard"
                                >
                                  {copiedPath === item.file_path ? "✓ Copied" : "📋 Copy"}
                                </button>
                                <button
                                  onClick={() => handleInspectMovie(item.id)}
                                  className="win9x-button text-[0.8125rem] !py-0.2 !px-2 text-blue-900"
                                  title="Inspect in Movie list"
                                >
                                  Inspect
                                </button>
                                <button
                                  onClick={() => startRenamePlan("movie", item.id)}
                                  className="win9x-button text-[0.8125rem] !py-0.2 !px-2 font-bold text-green-900"
                                  title="Preview dry-run rename according to Jellyfin multi-version standards"
                                >
                                  🏷 Rename
                                </button>
                                <button
                                  onClick={() => handleKeepOnlyMovie(item, group.items)}
                                  className="win9x-button text-[0.8125rem] !py-0.2 !px-2 font-bold text-blue-900"
                                  title="Keep this copy and archive other duplicates into _duplicates_archive/ (reversible via Undo)"
                                >
                                  ★ Keep Only This
                                </button>
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>

                  <div className="text-[0.8125rem] text-gray-600 flex items-center justify-between px-1">
                    <span>
                      💡 Tip: For multi-version movies, Jellyfin requires placing both files in the same folder with standard tags like `Blade Runner (1982) [imdbid-tt0083658] - 2160p Final Cut.mkv`.
                    </span>
                  </div>
                </div>
              );
            }))}
          </>
        )}

        {!isLoading && activeTab === "episodes" && (
          <>
            {episodeDuplicates.length === 0 ? (
              <div className="win9x-raised p-6 text-center bg-[#c0c0c0]">
                <div className="text-xl mb-1">🎉</div>
                <div className="font-bold text-base text-black mb-1">No Episode Duplicates Found!</div>
                <div className="text-[0.875rem] text-gray-700">
                  Every TV episode in your scanned libraries has a unique season/episode assignment.
                </div>
              </div>
            ) : (
              episodeDuplicates.map((group, gIdx) => (
                <div
                  key={`${group.show_title}-S${group.season}E${group.episode}-${gIdx}`}
                  className="win9x-raised p-2 bg-[#d4d0c8] space-y-1.5"
                >
                  {/* Group Header */}
                  <div className="flex items-center justify-between border-b border-[#808080] pb-1">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-[0.875rem] text-black">
                        {group.show_title}
                      </span>
                      <span className="font-mono text-[0.875rem] font-bold text-blue-900">
                        S{group.season.toString().padStart(2, "0")}E{group.episode.toString().padStart(2, "0")}
                      </span>
                      <span className="bg-[#000080] text-white text-[0.8125rem] font-mono px-1.5 py-0.5">
                        {group.match_key}
                      </span>
                    </div>
                    <span className="text-[0.8125rem] text-gray-700 font-bold">
                      {group.items.length} copies found
                    </span>
                  </div>

                  {/* Candidate Items Table */}
                  <div className="win9x-inset bg-white p-1 overflow-x-auto">
                    <table className="w-full text-[0.875rem] border-collapse">
                      <thead>
                        <tr className="bg-[#c0c0c0] border-b border-[#808080] text-left">
                          <th className="p-1.5 w-24">Quality</th>
                          <th className="p-1.5">File Path</th>
                          <th className="p-1.5 w-28">Resolution</th>
                          <th className="p-1.5 w-28">Size</th>
                          <th className="p-1.5 w-60 text-right">Actions</th>
                        </tr>
                      </thead>
                      <tbody>
                        {group.items.map((item: DuplicateEpisodeItem, idx: number) => {
                          const isPrimary = idx === 0;
                          return (
                            <tr
                              key={item.id}
                              className={`border-b border-[#e8e8e8] ${
                                isPrimary ? "bg-[#f5f9ff]" : "hover:bg-[#f9f9f9]"
                              }`}
                            >
                              <td className="p-1.5 font-bold">
                                {isPrimary ? (
                                  <span className="text-blue-800 text-[0.8125rem] flex items-center gap-0.5">
                                    ★ Preferred
                                  </span>
                                ) : (
                                  <span className="text-gray-500 text-[0.8125rem]">Duplicate #{idx + 1}</span>
                                )}
                              </td>
                              <td className="p-1.5 font-mono text-[0.8125rem] text-gray-800 break-all">
                                {item.file_path}
                              </td>
                              <td className="p-1.5 font-mono text-[0.8125rem]">
                                {item.resolution || "Unknown"}
                              </td>
                              <td className="p-1.5 font-mono text-[0.8125rem]">
                                {formatFileSize(item.file_size)}
                              </td>
                              <td className="p-1.5 text-right space-x-1.5 whitespace-nowrap">
                                <button
                                  onClick={() => handleCopyPath(item.file_path)}
                                  className="win9x-button text-[0.8125rem] !py-0.2 !px-2"
                                  title="Copy file path to clipboard"
                                >
                                  {copiedPath === item.file_path ? "✓ Copied" : "📋 Copy"}
                                </button>
                                <button
                                  onClick={() => handleKeepOnlyEpisode(item, group.items)}
                                  className="win9x-button text-[0.8125rem] !py-0.2 !px-2 font-bold text-blue-900"
                                  title="Keep this copy and archive other duplicates into _duplicates_archive/ (reversible via Undo)"
                                >
                                  ★ Keep Only This
                                </button>
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                </div>
              ))
            )}
          </>
        )}
      </div>
    </div>
  );
};
