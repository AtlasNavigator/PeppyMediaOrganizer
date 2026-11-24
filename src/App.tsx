import React, { useEffect } from "react";
import { Sidebar } from "./components/Sidebar/Sidebar";
import { Toolbar } from "./components/Toolbar/Toolbar";
import { MediaTable } from "./components/MediaTable/MediaTable";
import { PosterGrid } from "./components/PosterGrid/PosterGrid";
import { BatchBar } from "./components/BatchBar/BatchBar";
import { DuplicatesView } from "./components/DuplicatesView/DuplicatesView";
import { DetailPane } from "./components/DetailPane/DetailPane";
import { StatusBar } from "./components/StatusBar/StatusBar";
import { SettingsView } from "./components/Settings/SettingsView";
import { RenameModal } from "./components/RenameModal/RenameModal";
import { ImdbNotice } from "./components/ImdbNotice/ImdbNotice";
import { useMediaStore } from "./hooks/useMediaStore";
import { useScanProgress } from "./hooks/useScanProgress";
import { listen } from "@tauri-apps/api/event";

export const App: React.FC = () => {
  const activeSection = useMediaStore((s) => s.activeSection);
  const setActiveSection = useMediaStore((s) => s.setActiveSection);
  const viewMode = useMediaStore((s) => s.viewMode);
  const toggleViewMode = useMediaStore((s) => s.toggleViewMode);
  const loadMovies = useMediaStore((s) => s.loadMovies);
  const loadTvShows = useMediaStore((s) => s.loadTvShows);
  const loadLibraryRoots = useMediaStore((s) => s.loadLibraryRoots);
  const triggerScan = useMediaStore((s) => s.triggerScan);

  const isRenameModalOpen = useMediaStore((s) => s.isRenameModalOpen);
  const renamePlan = useMediaStore((s) => s.renamePlan);
  const isLoadingRenamePlan = useMediaStore((s) => s.isLoadingRenamePlan);
  const closeRenameModal = useMediaStore((s) => s.closeRenameModal);
  const checkCanUndo = useMediaStore((s) => s.checkCanUndo);
  const loadWatcherStatus = useMediaStore((s) => s.loadWatcherStatus);
  const loadDuplicates = useMediaStore((s) => s.loadDuplicates);
  const uiScale = useMediaStore((s) => s.uiScale);
  const initUiScale = useMediaStore((s) => s.initUiScale);
  const setUiScale = useMediaStore((s) => s.setUiScale);
  const loadImdbStatus = useMediaStore((s) => s.loadImdbStatus);
  const checkAndAutoRefreshImdb = useMediaStore((s) => s.checkAndAutoRefreshImdb);

  // Hook into scan events from backend
  useScanProgress();

  // Load initial data
  useEffect(() => {
    loadLibraryRoots();
    loadMovies();
    loadTvShows();
    checkCanUndo();
    loadWatcherStatus();
    initUiScale();
    loadImdbStatus();
    checkAndAutoRefreshImdb();
  }, [
    loadLibraryRoots,
    loadMovies,
    loadTvShows,
    checkCanUndo,
    loadWatcherStatus,
    initUiScale,
    loadImdbStatus,
    checkAndAutoRefreshImdb,
  ]);

  // Listen to IMDb progress events globally
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<any>("imdb:index_progress", (event) => {
      const payload = event.payload;
      if (payload) {
        useMediaStore.setState({
          imdbSyncProgress: payload.message,
          isImdbSyncing: !payload.is_finished,
        });
        if (payload.is_finished) {
          loadImdbStatus();
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [loadImdbStatus]);

  // Global keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+= / Ctrl++: Zoom in
      if (e.ctrlKey && (e.key === "=" || e.key === "+")) {
        e.preventDefault();
        setUiScale(uiScale + 0.1);
      }
      // Ctrl+-: Zoom out
      else if (e.ctrlKey && e.key === "-") {
        e.preventDefault();
        setUiScale(uiScale - 0.1);
      }
      // Ctrl+0: Reset zoom
      else if (e.ctrlKey && e.key === "0") {
        e.preventDefault();
        setUiScale(1.0);
      }
      // Ctrl+1: Movies
      else if (e.ctrlKey && e.key === "1") {
        e.preventDefault();
        setActiveSection("movies");
      }
      // Ctrl+2: TV Shows
      else if (e.ctrlKey && e.key === "2") {
        e.preventDefault();
        setActiveSection("tvshows");
      }
      // Ctrl+3: Settings
      else if (e.ctrlKey && e.key === "3") {
        e.preventDefault();
        setActiveSection("settings");
      }
      // Ctrl+4: Duplicates
      else if (e.ctrlKey && e.key === "4") {
        e.preventDefault();
        setActiveSection("duplicates");
      }
      // Ctrl+G: Toggle View Mode (Table vs Poster Grid)
      else if (e.ctrlKey && (e.key === "g" || e.key === "G")) {
        e.preventDefault();
        toggleViewMode();
      }
      // Ctrl+R: Rescan
      else if (e.ctrlKey && (e.key === "r" || e.key === "R")) {
        e.preventDefault();
        triggerScan();
      }
      // /: Focus search
      else if (e.key === "/" && document.activeElement?.tagName !== "INPUT") {
        e.preventDefault();
        const searchInput = document.getElementById("search-input");
        searchInput?.focus();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setActiveSection, toggleViewMode, triggerScan, uiScale, setUiScale]);

  return (
    <div className="w-full h-full flex flex-col bg-[#c0c0c0] overflow-hidden select-none">
      {/* Startup & Ingestion Non-Blocking Notice */}
      <ImdbNotice />

      {/* 3-Column Shell */}
      <div className="flex-1 flex flex-row min-h-0 overflow-hidden p-1 gap-1">
        {/* Left Column: Sidebar */}
        <Sidebar />

        {/* Center Column: Content */}
        {activeSection === "settings" ? (
          <SettingsView />
        ) : activeSection === "duplicates" ? (
          <main className="flex-1 flex flex-col gap-1 min-w-0 overflow-hidden">
            <Toolbar />
            <DuplicatesView />
          </main>
        ) : (
          <main className="flex-1 flex flex-col gap-1 min-w-0 overflow-hidden">
            <Toolbar />
            {viewMode === "table" ? <MediaTable /> : <PosterGrid />}
            <BatchBar />
          </main>
        )}

        {/* Right Column: Detail Pane (only visible in movies / tvshows media views) */}
        {(activeSection === "movies" || activeSection === "tvshows") && <DetailPane />}
      </div>

      {/* Bottom Column: Status Bar */}
      <StatusBar />

      {/* Phase 3 Dry-Run Rename Modal */}
      <RenameModal
        isOpen={isRenameModalOpen}
        plan={renamePlan}
        isLoading={isLoadingRenamePlan}
        onClose={closeRenameModal}
        onExecuted={() => {
          checkCanUndo();
          loadDuplicates();
        }}
      />
    </div>
  );
};

export default App;
