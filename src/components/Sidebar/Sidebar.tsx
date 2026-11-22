import React from "react";
import { useMediaStore } from "../../hooks/useMediaStore";

export const Sidebar: React.FC = () => {
  const activeSection = useMediaStore((s) => s.activeSection);
  const setActiveSection = useMediaStore((s) => s.setActiveSection);
  const movies = useMediaStore((s) => s.movies);
  const tvShows = useMediaStore((s) => s.tvShows);

  return (
    <aside className="w-[14.5rem] flex-shrink-0 flex flex-col bg-[#c0c0c0] win9x-raised p-1.5 select-none">
      <div className="win9x-titlebar mb-1.5 px-1.5 flex items-center justify-between">
        <span>Libraries</span>
      </div>

      <nav className="flex-1 flex flex-col gap-1">
        <button
          onClick={() => setActiveSection("movies")}
          className={`w-full text-left flex items-center justify-between px-2.5 py-1.5 cursor-pointer border-none outline-none ${
            activeSection === "movies"
              ? "bg-[#000080] text-white font-bold"
              : "bg-transparent text-black hover:bg-[#dfdfdf]"
          }`}
        >
          <span className="flex items-center gap-2">
            <span>🎬</span>
            <span>Movies</span>
          </span>
          <span className="text-[0.8125rem] opacity-80 font-mono">
            {movies.length > 0 ? `(${movies.length})` : "Ctrl+1"}
          </span>
        </button>

        <button
          onClick={() => setActiveSection("tvshows")}
          className={`w-full text-left flex items-center justify-between px-2.5 py-1.5 cursor-pointer border-none outline-none ${
            activeSection === "tvshows"
              ? "bg-[#000080] text-white font-bold"
              : "bg-transparent text-black hover:bg-[#dfdfdf]"
          }`}
        >
          <span className="flex items-center gap-2">
            <span>📺</span>
            <span>TV Shows</span>
          </span>
          <span className="text-[0.8125rem] opacity-80 font-mono">
            {tvShows.length > 0 ? `(${tvShows.length})` : "Ctrl+2"}
          </span>
        </button>

        <button
          onClick={() => setActiveSection("duplicates")}
          className={`w-full text-left flex items-center justify-between px-2.5 py-1.5 cursor-pointer border-none outline-none ${
            activeSection === "duplicates"
              ? "bg-[#000080] text-white font-bold"
              : "bg-transparent text-black hover:bg-[#dfdfdf]"
          }`}
        >
          <span className="flex items-center gap-2">
            <span>👥</span>
            <span>Duplicates</span>
          </span>
          <span className="text-[0.8125rem] opacity-80 font-mono">Ctrl+4</span>
        </button>

        <div className="my-1 border-t border-[#808080] border-b border-[#ffffff]" />

        <button
          onClick={() => setActiveSection("settings")}
          className={`w-full text-left flex items-center justify-between px-2.5 py-1.5 cursor-pointer border-none outline-none ${
            activeSection === "settings"
              ? "bg-[#000080] text-white font-bold"
              : "bg-transparent text-black hover:bg-[#dfdfdf]"
          }`}
        >
          <span className="flex items-center gap-2">
            <span>⚙️</span>
            <span>Settings</span>
          </span>
          <span className="text-[0.8125rem] opacity-80 font-mono">Ctrl+3</span>
        </button>
      </nav>

      {/* Library info box */}
      <div className="win9x-panel-inset p-2 mt-auto text-[0.8125rem] text-[#404040] space-y-0.5">
        <div className="font-bold text-black mb-1">Jellyfin Standard</div>
        <div>Strict [imdbid-tt...]</div>
        <div>Zero-pad Seasons</div>
        <div>UTF-8 NFO XML</div>
      </div>
    </aside>
  );
};
