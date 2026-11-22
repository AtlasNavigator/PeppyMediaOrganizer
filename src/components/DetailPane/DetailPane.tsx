import React, { useState } from "react";
import { useMediaStore } from "../../hooks/useMediaStore";
import { ScrapeModal } from "../ScrapeModal/ScrapeModal";

export const DetailPane: React.FC = () => {
  const activeSection = useMediaStore((s) => s.activeSection);
  const movies = useMediaStore((s) => s.movies);
  const selectedMovieId = useMediaStore((s) => s.selectedMovieId);

  const tvShows = useMediaStore((s) => s.tvShows);
  const selectedShowId = useMediaStore((s) => s.selectedShowId);
  const episodes = useMediaStore((s) => s.episodes);
  const startRenamePlan = useMediaStore((s) => s.startRenamePlan);

  const activeStreamInfo = useMediaStore((s) => s.activeStreamInfo);
  const isLoadingStreamInfo = useMediaStore((s) => s.isLoadingStreamInfo);
  const probeCurrentItem = useMediaStore((s) => s.probeCurrentItem);
  const loadStreamInfo = useMediaStore((s) => s.loadStreamInfo);

  const [activeTab, setActiveTab] = useState<"details" | "episodes">("details");
  const [selectedEpisodeId, setSelectedEpisodeId] = useState<number | null>(null);
  const [isScrapeOpen, setIsScrapeOpen] = useState(false);

  const movie = movies.find((m) => m.id === selectedMovieId);
  const show = tvShows.find((s) => s.id === selectedShowId);

  const formatFileSize = (bytes?: number | null) => {
    if (!bytes || bytes === 0) return "0 B (Mock / Stub)";
    const mb = bytes / (1024 * 1024);
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(2)} GB`;
    }
    return `${mb.toFixed(1)} MB`;
  };

  const formatDuration = (secs?: number | null) => {
    if (!secs || secs <= 0) return "Unknown";
    const totalSecs = Math.round(secs);
    const hours = Math.floor(totalSecs / 3600);
    const minutes = Math.floor((totalSecs % 3600) / 60);
    const remainingSecs = totalSecs % 60;
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m ${remainingSecs}s`;
  };

  if (activeSection === "movies") {
    if (!movie) {
      return (
        <aside className="w-[22rem] flex-shrink-0 win9x-raised p-2 flex items-center justify-center text-[#808080] text-[0.875rem] select-none">
          Select a movie to inspect metadata
        </aside>
      );
    }

    return (
      <aside className="w-[22rem] flex-shrink-0 win9x-raised flex flex-col p-1.5 overflow-hidden select-none">
        <div className="win9x-titlebar mb-1.5 px-1.5 flex items-center justify-between">
          <span>Metadata & Sidecars</span>
          <span className="text-[0.8125rem] opacity-75 font-mono">#{movie.id}</span>
        </div>

        <div className="flex-1 overflow-y-auto space-y-2 p-1 text-[0.875rem]">
          {/* Header Card */}
          <div className="win9x-panel-inset p-2">
            <div className="flex items-start justify-between gap-2">
              <div>
                <h3 className="font-bold text-[1rem] text-black break-words leading-tight">
                  {movie.title}
                </h3>
                <div className="text-[#505050] text-[0.8125rem] mt-0.5">
                  {movie.year ? `(${movie.year})` : "Year Unknown"}
                  {movie.resolution && ` · ${movie.resolution}`}
                  {movie.version_label && ` · ${movie.version_label}`}
                </div>
              </div>
              <div className="flex items-center gap-1 flex-shrink-0">
                <button
                  onClick={() => setIsScrapeOpen(true)}
                  className="win9x-button text-[0.8125rem] font-bold text-blue-900 whitespace-nowrap"
                  title="Search and scrape metadata from IMDb, TMDb, TVDb, OMDb"
                >
                  🔍 Scrape
                </button>
                <button
                  onClick={() => startRenamePlan("movie", movie.id)}
                  className="win9x-button text-[0.8125rem] font-bold text-green-900 whitespace-nowrap"
                  title="Preview dry-run rename for this movie and its sidecars"
                >
                  🏷 Rename
                </button>
              </div>
            </div>
            {movie.plot && (
              <p className="text-[0.8125rem] text-gray-700 mt-2 leading-relaxed border-t border-gray-300 pt-1.5 max-h-28 overflow-y-auto">
                {movie.plot}
              </p>
            )}
          </div>

          {/* Provider IDs Section */}
          <div className="win9x-raised p-2 bg-[#c0c0c0]">
            <div className="font-bold border-b border-[#808080] pb-1 mb-1.5 flex items-center justify-between">
              <span>Provider IDs</span>
              <span className="text-[0.75rem] text-[#606060]">Jellyfin Standard</span>
            </div>

            <div className="space-y-1">
              <div className="flex items-center justify-between">
                <span className="font-medium">IMDb ID:</span>
                {movie.imdb_id ? (
                  <span className="font-mono text-green-800 font-bold bg-green-50 px-1.5 py-0.2 border border-green-300 text-[0.8125rem]">
                    ✓ {movie.imdb_id}
                  </span>
                ) : (
                  <span className="text-red-700 text-[0.8125rem]">✗ Missing</span>
                )}
              </div>

              <div className="flex items-center justify-between">
                <span className="font-medium">TMDb ID:</span>
                {movie.tmdb_id ? (
                  <span className="font-mono text-green-800 font-bold bg-green-50 px-1.5 py-0.2 border border-green-300 text-[0.8125rem]">
                    ✓ {movie.tmdb_id}
                  </span>
                ) : (
                  <span className="text-red-700 text-[0.8125rem]">✗ Missing</span>
                )}
              </div>
            </div>
          </div>

          {/* Phase 5: Technical Media Stream Info (ffprobe) */}
          <div className="win9x-raised p-2 bg-[#c0c0c0]">
            <div className="font-bold border-b border-[#808080] pb-1 mb-1.5 flex items-center justify-between">
              <span>Technical Stream Info</span>
              <button
                onClick={() => probeCurrentItem(movie.file_path, "movie", movie.id)}
                disabled={isLoadingStreamInfo}
                className="win9x-button text-[0.75rem] !py-0.2 !px-1.5 font-bold text-blue-900"
                title="Run ffprobe inspection on this file"
              >
                {isLoadingStreamInfo ? "Probing..." : activeStreamInfo ? "⟳ Re-probe" : "⚡ ffprobe"}
              </button>
            </div>

            {isLoadingStreamInfo ? (
              <div className="text-[0.8125rem] text-gray-600 p-2 text-center">
                Running ffprobe stream analysis...
              </div>
            ) : activeStreamInfo ? (
              <div className="space-y-1.5 text-[0.8125rem]">
                {/* Badges row */}
                <div className="flex flex-wrap gap-1">
                  {activeStreamInfo.video_codec && (
                    <span className="bg-[#2c3e50] text-white px-1.5 py-0.5 font-mono font-bold text-[0.75rem]">
                      {activeStreamInfo.video_codec.toUpperCase()}
                    </span>
                  )}
                  {activeStreamInfo.hdr_format && activeStreamInfo.hdr_format !== "SDR" ? (
                    <span className="bg-[#8e44ad] text-white px-1.5 py-0.5 font-mono font-bold text-[0.75rem]">
                      {activeStreamInfo.hdr_format}
                    </span>
                  ) : (
                    <span className="bg-[#7f8c8d] text-white px-1.5 py-0.5 font-mono text-[0.75rem]">
                      SDR
                    </span>
                  )}
                  {activeStreamInfo.width && activeStreamInfo.height && (
                    <span className="bg-[#27ae60] text-white px-1.5 py-0.5 font-mono font-bold text-[0.75rem]">
                      {activeStreamInfo.width >= 3800
                        ? "4K UHD"
                        : activeStreamInfo.width >= 1900
                        ? "1080p"
                        : activeStreamInfo.width >= 1200
                        ? "720p"
                        : `${activeStreamInfo.width}x${activeStreamInfo.height}`}
                    </span>
                  )}
                  {activeStreamInfo.audio_codec && (
                    <span className="bg-[#d35400] text-white px-1.5 py-0.5 font-mono font-bold text-[0.75rem]">
                      {activeStreamInfo.audio_codec.toUpperCase()}{" "}
                      {activeStreamInfo.audio_channels ? `${activeStreamInfo.audio_channels}ch` : ""}
                    </span>
                  )}
                </div>

                <div className="win9x-panel-inset p-1.5 space-y-1">
                  <div className="flex justify-between">
                    <span className="text-gray-600">Container:</span>
                    <span className="font-mono font-bold">
                      {activeStreamInfo.container ? activeStreamInfo.container.toUpperCase() : "Unknown"}
                    </span>
                  </div>
                  {activeStreamInfo.width && activeStreamInfo.height && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Dimensions:</span>
                      <span className="font-mono">
                        {activeStreamInfo.width} × {activeStreamInfo.height}
                      </span>
                    </div>
                  )}
                  {activeStreamInfo.bitrate_kbps && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Bitrate:</span>
                      <span className="font-mono">
                        {(activeStreamInfo.bitrate_kbps / 1000).toFixed(2)} Mbps
                      </span>
                    </div>
                  )}
                  {activeStreamInfo.duration_sec && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Duration:</span>
                      <span className="font-mono">
                        {formatDuration(activeStreamInfo.duration_sec)}
                      </span>
                    </div>
                  )}
                  {activeStreamInfo.audio_layout && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Audio Layout:</span>
                      <span className="font-mono">{activeStreamInfo.audio_layout}</span>
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="text-[0.8125rem] text-gray-500 win9x-panel-inset p-1.5 flex items-center justify-between">
                <span>No technical stream info probed yet.</span>
                <button
                  onClick={() => probeCurrentItem(movie.file_path, "movie", movie.id)}
                  className="win9x-button text-[0.75rem] !py-0.2 !px-1.5 font-bold"
                >
                  ⚡ Probe
                </button>
              </div>
            )}
          </div>

          {/* Sidecars & Assets */}
          <div className="win9x-raised p-2 bg-[#c0c0c0]">
            <div className="font-bold border-b border-[#808080] pb-1 mb-1.5">
              Local Filesystem Sidecars
            </div>

            <div className="grid grid-cols-2 gap-1.5 text-[0.8125rem]">
              <div className="win9x-panel-inset p-1.5 flex items-center justify-between">
                <span>movie.nfo</span>
                <span className={movie.has_nfo ? "text-green-700 font-bold" : "text-gray-400"}>
                  {movie.has_nfo ? "✓ Found" : "✗ None"}
                </span>
              </div>

              <div className="win9x-panel-inset p-1.5 flex items-center justify-between">
                <span>poster.jpg</span>
                <span className={movie.has_poster ? "text-green-700 font-bold" : "text-gray-400"}>
                  {movie.has_poster ? "✓ Found" : "✗ None"}
                </span>
              </div>

              <div className="win9x-panel-inset p-1.5 flex items-center justify-between">
                <span>backdrop.jpg</span>
                <span className={movie.has_backdrop ? "text-green-700 font-bold" : "text-gray-400"}>
                  {movie.has_backdrop ? "✓ Found" : "✗ None"}
                </span>
              </div>

              <div className="win9x-panel-inset p-1.5 flex items-center justify-between">
                <span>Subtitles</span>
                <span className={movie.subtitle_count > 0 ? "text-purple-700 font-bold" : "text-gray-400"}>
                  {movie.subtitle_count > 0 ? `${movie.subtitle_count} stream(s)` : "None"}
                </span>
              </div>
            </div>
          </div>

          {/* File Paths */}
          <div className="win9x-raised p-2 bg-[#c0c0c0]">
            <div className="font-bold border-b border-[#808080] pb-1 mb-1.5">
              Filesystem Location
            </div>

            <div className="space-y-1.5">
              <div>
                <span className="text-[0.8125rem] text-[#505050] font-bold">Directory:</span>
                <div className="win9x-input font-mono text-[0.8125rem] break-all max-h-20 overflow-y-auto">
                  {movie.folder_path}
                </div>
              </div>

              <div>
                <span className="text-[0.8125rem] text-[#505050] font-bold">Video File:</span>
                <div className="win9x-input font-mono text-[0.8125rem] break-all max-h-20 overflow-y-auto">
                  {movie.file_path}
                </div>
              </div>

              <div className="flex justify-between text-[0.8125rem] pt-1">
                <span className="text-[#505050]">File Size:</span>
                <span className="font-mono">{formatFileSize(movie.file_size)}</span>
              </div>
            </div>
          </div>
        </div>
        <ScrapeModal isOpen={isScrapeOpen} onClose={() => setIsScrapeOpen(false)} />
      </aside>
    );
  } else {
    // TV Shows Detail
    if (!show) {
      return (
        <aside className="w-[22rem] flex-shrink-0 win9x-raised p-2 flex items-center justify-center text-[#808080] text-[0.875rem] select-none">
          Select a TV show to inspect episodes
        </aside>
      );
    }

    return (
      <aside className="w-[22rem] flex-shrink-0 win9x-raised flex flex-col p-1.5 overflow-hidden select-none">
        <div className="win9x-titlebar mb-1.5 px-1.5 flex items-center justify-between">
          <span>Series Inspector</span>
          <span className="text-[0.8125rem] opacity-75 font-mono">#{show.id}</span>
        </div>

        {/* Tab Controls */}
        <div className="flex border-b border-[#808080] px-1 mb-1">
          <button
            onClick={() => setActiveTab("details")}
            className={`win9x-tab ${activeTab === "details" ? "active" : ""}`}
          >
            Series Details
          </button>
          <button
            onClick={() => setActiveTab("episodes")}
            className={`win9x-tab ${activeTab === "episodes" ? "active" : ""}`}
          >
            Episodes ({episodes.length})
          </button>
        </div>

        {activeTab === "details" ? (
          <div className="flex-1 overflow-y-auto space-y-2 p-1 text-[0.875rem]">
            <div className="win9x-panel-inset p-2">
              <div className="flex items-start justify-between gap-2">
                <div>
                  <h3 className="font-bold text-[1rem] text-black break-words leading-tight">
                    {show.title}
                  </h3>
                  <div className="text-[#505050] text-[0.8125rem] mt-0.5">
                    {show.year ? `(${show.year})` : "Year Unknown"}
                    {show.status && ` · ${show.status}`}
                  </div>
                </div>
                <div className="flex items-center gap-1 flex-shrink-0">
                  <button
                    onClick={() => setIsScrapeOpen(true)}
                    className="win9x-button text-[0.8125rem] font-bold text-blue-900 whitespace-nowrap"
                    title="Search TV metadata and remap episode ordering"
                  >
                    🔍 Scrape / Order
                  </button>
                  <button
                    onClick={() => startRenamePlan("tvshow", show.id)}
                    className="win9x-button text-[0.8125rem] font-bold text-green-900 whitespace-nowrap"
                    title="Preview dry-run rename for this show, seasons, episodes, and sidecars"
                  >
                    🏷 Rename
                  </button>
                </div>
              </div>
            </div>

            {/* Provider IDs */}
            <div className="win9x-raised p-2 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-1.5">
                TV Series Provider IDs
              </div>

              <div className="space-y-1">
                <div className="flex items-center justify-between">
                  <span>TVDb ID:</span>
                  {show.tvdb_id ? (
                    <span className="font-mono text-green-800 font-bold bg-green-50 px-1.5 py-0.2 border border-green-300 text-[0.8125rem]">
                      ✓ {show.tvdb_id}
                    </span>
                  ) : (
                    <span className="text-red-700 text-[0.8125rem]">✗ Missing</span>
                  )}
                </div>

                <div className="flex items-center justify-between">
                  <span>TMDb ID:</span>
                  {show.tmdb_id ? (
                    <span className="font-mono text-green-800 font-bold bg-green-50 px-1.5 py-0.2 border border-green-300 text-[0.8125rem]">
                      ✓ {show.tmdb_id}
                    </span>
                  ) : (
                    <span className="text-red-700 text-[0.8125rem]">✗ Missing</span>
                  )}
                </div>

                <div className="flex items-center justify-between">
                  <span>IMDb ID:</span>
                  {show.imdb_id ? (
                    <span className="font-mono text-green-800 font-bold bg-green-50 px-1.5 py-0.2 border border-green-300 text-[0.8125rem]">
                      ✓ {show.imdb_id}
                    </span>
                  ) : (
                    <span className="text-red-700 text-[0.8125rem]">✗ Missing</span>
                  )}
                </div>
              </div>
            </div>

            {/* Sidecars */}
            <div className="win9x-raised p-2 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-1.5">
                Series Assets
              </div>
              <div className="grid grid-cols-2 gap-1.5 text-[0.8125rem]">
                <div className="win9x-panel-inset p-1.5 flex justify-between">
                  <span>tvshow.nfo</span>
                  <span className={show.has_nfo ? "text-green-700 font-bold" : "text-gray-400"}>
                    {show.has_nfo ? "✓ Yes" : "✗ None"}
                  </span>
                </div>
                <div className="win9x-panel-inset p-1.5 flex justify-between">
                  <span>poster.jpg</span>
                  <span className={show.has_poster ? "text-green-700 font-bold" : "text-gray-400"}>
                    {show.has_poster ? "✓ Yes" : "✗ None"}
                  </span>
                </div>
              </div>
            </div>

            {/* Location */}
            <div className="win9x-raised p-2 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-1">
                Series Directory
              </div>
              <div className="win9x-input font-mono text-[0.8125rem] break-all max-h-20 overflow-y-auto">
                {show.folder_path}
              </div>
            </div>
          </div>
        ) : (
          <div className="flex-1 overflow-y-auto win9x-inset p-1.5 space-y-1.5">
            {episodes.length === 0 ? (
              <div className="p-4 text-center text-gray-500 text-[0.875rem]">
                No episode files scanned yet.
              </div>
            ) : (
              episodes.map((ep) => (
                <div
                  key={ep.id}
                  onClick={() => {
                    if (selectedEpisodeId === ep.id) {
                      setSelectedEpisodeId(null);
                    } else {
                      setSelectedEpisodeId(ep.id);
                      loadStreamInfo("episode", ep.id);
                    }
                  }}
                  className={`p-2 border-b border-[#d0d0d0] cursor-pointer text-[0.875rem] ${
                    selectedEpisodeId === ep.id ? "bg-[#000080] text-white" : "hover:bg-[#f0f0f0] bg-white"
                  }`}
                >
                  <div className="flex items-center justify-between font-bold">
                    <span className={`font-mono ${selectedEpisodeId === ep.id ? "text-white" : "text-blue-900"}`}>
                      S{ep.season.toString().padStart(2, "0")}E{ep.episode.toString().padStart(2, "0")}
                      {ep.episode_end ? `-E${ep.episode_end.toString().padStart(2, "0")}` : ""}
                    </span>
                    <span className={`text-[0.8125rem] font-normal ${selectedEpisodeId === ep.id ? "text-gray-200" : "text-gray-500"}`}>
                      {ep.resolution || ""}
                    </span>
                  </div>
                  <div className="truncate mt-0.5">
                    {ep.title || "Episode"}
                  </div>
                  <div className={`flex items-center justify-between text-[0.8125rem] mt-1 ${selectedEpisodeId === ep.id ? "text-gray-300" : "text-[#606060]"}`}>
                    <span>{ep.has_nfo ? "✓ NFO" : "No NFO"}</span>
                    <span>{ep.subtitle_count > 0 ? `✓ Subs (${ep.subtitle_count})` : "No subs"}</span>
                  </div>

                  {/* Expanded Technical Stream Info for selected episode */}
                  {selectedEpisodeId === ep.id && (
                    <div
                      className="mt-2 p-1.5 win9x-inset bg-[#e8e8e8] text-black text-[0.8125rem]"
                      onClick={(e) => e.stopPropagation()}
                    >
                      <div className="flex items-center justify-between font-bold mb-1 border-b border-[#a0a0a0] pb-0.5">
                        <span>ffprobe Streams</span>
                        <button
                          onClick={() => probeCurrentItem(ep.file_path, "episode", ep.id)}
                          disabled={isLoadingStreamInfo}
                          className="win9x-button text-[0.75rem] !py-0.2 !px-1.5 font-bold text-blue-900"
                        >
                          {isLoadingStreamInfo ? "Probing..." : activeStreamInfo ? "⟳ Probe" : "⚡ ffprobe"}
                        </button>
                      </div>
                      {isLoadingStreamInfo ? (
                        <div className="text-gray-600 text-center py-1">Running ffprobe...</div>
                      ) : activeStreamInfo ? (
                        <div className="space-y-1">
                          <div className="flex flex-wrap gap-1 mb-1">
                            {activeStreamInfo.video_codec && (
                              <span className="bg-[#2c3e50] text-white px-1.5 py-0.2 font-mono font-bold text-[0.75rem]">
                                {activeStreamInfo.video_codec.toUpperCase()}
                              </span>
                            )}
                            {activeStreamInfo.hdr_format && activeStreamInfo.hdr_format !== "SDR" && (
                              <span className="bg-[#8e44ad] text-white px-1.5 py-0.2 font-mono font-bold text-[0.75rem]">
                                {activeStreamInfo.hdr_format}
                              </span>
                            )}
                            {activeStreamInfo.audio_codec && (
                              <span className="bg-[#d35400] text-white px-1.5 py-0.2 font-mono font-bold text-[0.75rem]">
                                {activeStreamInfo.audio_codec.toUpperCase()}{" "}
                                {activeStreamInfo.audio_channels ? `${activeStreamInfo.audio_channels}ch` : ""}
                              </span>
                            )}
                          </div>
                          <div className="flex justify-between">
                            <span className="text-gray-600">Container:</span>
                            <span className="font-mono font-bold">{activeStreamInfo.container || "Unknown"}</span>
                          </div>
                          {activeStreamInfo.bitrate_kbps && (
                            <div className="flex justify-between">
                              <span className="text-gray-600">Bitrate:</span>
                              <span className="font-mono">{(activeStreamInfo.bitrate_kbps / 1000).toFixed(2)} Mbps</span>
                            </div>
                          )}
                          {activeStreamInfo.duration_sec && (
                            <div className="flex justify-between">
                              <span className="text-gray-600">Duration:</span>
                              <span className="font-mono">{formatDuration(activeStreamInfo.duration_sec)}</span>
                            </div>
                          )}
                        </div>
                      ) : (
                        <div className="text-gray-500 italic">Not probed yet.</div>
                      )}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        )}

        <ScrapeModal isOpen={isScrapeOpen} onClose={() => setIsScrapeOpen(false)} />
      </aside>
    );
  }
};
