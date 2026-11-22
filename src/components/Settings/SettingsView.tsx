import React, { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useMediaStore } from "../../hooks/useMediaStore";
import { api } from "../../lib/tauri";
import type { DatasetUpdateInfo, ImdbSyncStatus, ProviderConfigDto } from "../../lib/types";

export const SettingsView: React.FC = () => {
  const libraryRoots = useMediaStore((s) => s.libraryRoots);
  const loadLibraryRoots = useMediaStore((s) => s.loadLibraryRoots);
  const triggerScan = useMediaStore((s) => s.triggerScan);
  const isScanning = useMediaStore((s) => s.isScanning);
  const watcherStatus = useMediaStore((s) => s.watcherStatus);
  const setWatcherEnabled = useMediaStore((s) => s.setWatcherEnabled);
  const uiScale = useMediaStore((s) => s.uiScale);
  const systemTextScale = useMediaStore((s) => s.systemTextScale);
  const setUiScale = useMediaStore((s) => s.setUiScale);

  const [activeTab, setActiveTab] = useState<"roots" | "display" | "providers" | "imdb" | "priorities" | "mediainfo" | "rules" | "about">("roots");

  // Library Roots State
  const [manualPath, setManualPath] = useState("");
  const [mediaType, setMediaType] = useState<"movie" | "tvshow">("movie");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  // Provider Configs State
  const [, setProviderConfigs] = useState<ProviderConfigDto[]>([]);
  const [tmdbKey, setTmdbKey] = useState("");
  const [tvdbKey, setTvdbKey] = useState("");
  const [tvdbPin, setTvdbPin] = useState("");
  const [omdbKey, setOmdbKey] = useState("");
  const [providersSavedMsg, setProvidersSavedMsg] = useState<string | null>(null);

  // Field Priorities State
  const [priorities, setPriorities] = useState<Record<string, string[]>>({});

  // ffprobe / Media Info State
  const [ffprobePath, setFfprobePath] = useState<string>("");
  const [ffprobeDetectedPath, setFfprobeDetectedPath] = useState<string | null>(null);
  const [ffprobeSavedMsg, setFfprobeSavedMsg] = useState<string | null>(null);
  const [isProbingBatch, setIsProbingBatch] = useState(false);
  const [probeBatchResult, setProbeBatchResult] = useState<string | null>(null);

  // IMDb Dataset State
  const [syncStatus, setSyncStatus] = useState<ImdbSyncStatus[]>([]);
  const [updateInfos, setUpdateInfos] = useState<DatasetUpdateInfo[]>([]);
  const [imdbFolderPath, setImdbFolderPath] = useState("/home/pepper/Documents/Antigravity Projects/PeppyMediaOrganizer");
  const [isIndexingImdb, setIsIndexingImdb] = useState(false);
  const [imdbStatusText, setImdbStatusText] = useState("");
  const [isCheckingUpdates, setIsCheckingUpdates] = useState(false);

  useEffect(() => {
    loadProviders();
    loadImdbStatus();
    loadPriorities();
    loadFfprobe();

    // Listen to indexing progress events
    let unlisten: (() => void) | undefined;
    listen<any>("imdb:index_progress", (event) => {
      setImdbStatusText(event.payload.message || `Indexed ${event.payload.rows_indexed} items...`);
      if (event.payload.is_finished) {
        setIsIndexingImdb(false);
        loadImdbStatus();
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const loadProviders = async () => {
    try {
      const configs = await api.getProviderConfigs();
      setProviderConfigs(configs);
      for (const c of configs) {
        if (c.provider_id === "tmdb") setTmdbKey(c.api_key || "");
        if (c.provider_id === "tvdb") {
          setTvdbKey(c.api_key || "");
          setTvdbPin(c.pin_or_token || "");
        }
        if (c.provider_id === "omdb") setOmdbKey(c.api_key || "");
      }
    } catch (err) {
      console.error("Failed to load provider configs:", err);
    }
  };

  const loadImdbStatus = async () => {
    try {
      const status = await api.getImdbSyncStatus();
      setSyncStatus(status);
    } catch (err) {
      console.error("Failed to load IMDb status:", err);
    }
  };

  const loadPriorities = async () => {
    try {
      const p = await api.getFieldPriorities();
      setPriorities(p);
    } catch (err) {
      console.error("Failed to load field priorities:", err);
    }
  };

  const loadFfprobe = async () => {
    try {
      const path = await api.getFfprobePath();
      setFfprobePath(path || "");
      setFfprobeDetectedPath(path || null);
    } catch (err) {
      console.error("Failed to load ffprobe path:", err);
      setFfprobeDetectedPath(null);
    }
  };

  const handleSaveFfprobe = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.setFfprobePath(ffprobePath.trim());
      setFfprobeSavedMsg("✅ ffprobe path saved!");
      setTimeout(() => setFfprobeSavedMsg(null), 2500);
      await loadFfprobe();
    } catch (err) {
      setFfprobeSavedMsg(`Error saving path: ${err}`);
    }
  };

  const handlePickFfprobeBinary = async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: "Select ffprobe Binary / Executable",
      });
      if (selected && typeof selected === "string") {
        setFfprobePath(selected);
      }
    } catch (err) {
      console.error("Binary picker error:", err);
    }
  };

  const handleRunBatchProbe = async () => {
    try {
      setIsProbingBatch(true);
      setProbeBatchResult(null);
      const count = await api.probePendingBatch(25);
      setProbeBatchResult(`Probed ${count} media files.`);
    } catch (err) {
      setProbeBatchResult(`Batch probe error: ${err}`);
    } finally {
      setIsProbingBatch(false);
    }
  };

  const handleSaveProviders = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.setProviderConfig("tmdb", true, tmdbKey.trim() || null, null);
      await api.setProviderConfig("tvdb", true, tvdbKey.trim() || null, tvdbPin.trim() || null);
      await api.setProviderConfig("omdb", true, omdbKey.trim() || null, null);
      setProvidersSavedMsg("✅ Provider API keys successfully saved!");
      setTimeout(() => setProvidersSavedMsg(null), 2500);
      await loadProviders();
    } catch (err) {
      setProvidersSavedMsg(`Error saving keys: ${err}`);
    }
  };

  const handleIndexLocalImdb = async () => {
    try {
      setIsIndexingImdb(true);
      setImdbStatusText("Starting decompression and streaming ingestion...");
      const res = await api.indexImdbFromLocal(imdbFolderPath.trim());
      setImdbStatusText(`✅ ${res}`);
      await loadImdbStatus();
    } catch (err) {
      setImdbStatusText(`❌ Ingestion failed: ${err}`);
    } finally {
      setIsIndexingImdb(false);
    }
  };

  const handleCheckUpstream = async () => {
    try {
      setIsCheckingUpdates(true);
      const updates = await api.checkImdbUpdates();
      setUpdateInfos(updates);
    } catch (err) {
      console.error("Failed to check upstream updates:", err);
    } finally {
      setIsCheckingUpdates(false);
    }
  };

  const handlePickFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Jellyfin Library Folder",
      });
      if (selected && typeof selected === "string") {
        setManualPath(selected);
      }
    } catch (err) {
      console.error("Folder picker error:", err);
    }
  };

  const handlePickImdbFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Folder Containing title.*.tsv.gz files",
      });
      if (selected && typeof selected === "string") {
        setImdbFolderPath(selected);
      }
    } catch (err) {
      console.error("IMDb folder picker error:", err);
    }
  };

  const handleAddRoot = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!manualPath.trim()) {
      setErrorMsg("Please choose or enter a directory path.");
      return;
    }
    try {
      setErrorMsg(null);
      await api.addLibraryRoot(manualPath.trim(), mediaType);
      setManualPath("");
      await loadLibraryRoots();
    } catch (err) {
      setErrorMsg(`Failed to add library: ${err}`);
    }
  };

  const handleToggle = async (id: number, enabled: boolean) => {
    try {
      await api.toggleLibraryRootEnabled(id, enabled);
      await loadLibraryRoots();
    } catch (err) {
      console.error("Toggle error:", err);
    }
  };

  const handleRemove = async (id: number) => {
    if (confirm("Are you sure you want to remove this library root?")) {
      try {
        await api.removeLibraryRoot(id);
        await loadLibraryRoots();
      } catch (err) {
        console.error("Remove error:", err);
      }
    }
  };

  return (
    <div className="flex-1 flex flex-col p-2 win9x-raised overflow-hidden select-none">
      {/* Tabs */}
      <div className="flex border-b border-[#808080] mb-2 px-1 flex-wrap">
        <button
          onClick={() => setActiveTab("roots")}
          className={`win9x-tab ${activeTab === "roots" ? "active" : ""}`}
        >
          Library Folders
        </button>
        <button
          onClick={() => setActiveTab("display")}
          className={`win9x-tab ${activeTab === "display" ? "active" : ""}`}
        >
          Display & Scaling
        </button>
        <button
          onClick={() => setActiveTab("providers")}
          className={`win9x-tab ${activeTab === "providers" ? "active" : ""}`}
        >
          API Providers & Keys
        </button>
        <button
          onClick={() => setActiveTab("imdb")}
          className={`win9x-tab ${activeTab === "imdb" ? "active" : ""}`}
        >
          IMDb Local Dataset
        </button>
        <button
          onClick={() => setActiveTab("priorities")}
          className={`win9x-tab ${activeTab === "priorities" ? "active" : ""}`}
        >
          Field Priorities
        </button>
        <button
          onClick={() => setActiveTab("mediainfo")}
          className={`win9x-tab ${activeTab === "mediainfo" ? "active" : ""}`}
        >
          ffprobe & Codecs
        </button>
        <button
          onClick={() => setActiveTab("rules")}
          className={`win9x-tab ${activeTab === "rules" ? "active" : ""}`}
        >
          Jellyfin Standards
        </button>
        <button
          onClick={() => setActiveTab("about")}
          className={`win9x-tab ${activeTab === "about" ? "active" : ""}`}
        >
          About
        </button>
      </div>

      {/* Tab Contents */}
      <div className="flex-1 win9x-panel-inset p-3 overflow-y-auto">
        {activeTab === "roots" && (
          <div className="space-y-4 max-w-3xl text-[0.875rem]">
            <div className="win9x-raised p-3 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-2">
                Add Library Directory
              </div>

              <form onSubmit={handleAddRoot} className="space-y-2">
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={manualPath}
                    onChange={(e) => setManualPath(e.target.value)}
                    placeholder="/path/to/media/library (or mounted share)"
                    className="win9x-input flex-1 font-mono text-[0.8125rem]"
                  />
                  <button
                    type="button"
                    onClick={handlePickFolder}
                    className="win9x-button text-[0.8125rem]"
                  >
                    Browse...
                  </button>
                </div>

                <div className="flex items-center justify-between pt-1">
                  <div className="flex items-center gap-4">
                    <label className="flex items-center gap-1.5 cursor-pointer">
                      <input
                        type="radio"
                        name="media_type"
                        checked={mediaType === "movie"}
                        onChange={() => setMediaType("movie")}
                      />
                      <span>Movies Library</span>
                    </label>

                    <label className="flex items-center gap-1.5 cursor-pointer">
                      <input
                        type="radio"
                        name="media_type"
                        checked={mediaType === "tvshow"}
                        onChange={() => setMediaType("tvshow")}
                      />
                      <span>TV Shows Library</span>
                    </label>
                  </div>

                  <button
                    type="submit"
                    className="win9x-button font-bold text-blue-900 text-[0.8125rem]"
                  >
                    + Add Library
                  </button>
                </div>

                {errorMsg && (
                  <div className="text-red-700 font-bold text-[0.8125rem] mt-1">
                    {errorMsg}
                  </div>
                )}
              </form>
            </div>

            <div className="win9x-raised p-3 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-2 flex items-center justify-between">
                <span>Configured Library Folders ({libraryRoots.length})</span>
                <button
                  onClick={() => triggerScan()}
                  disabled={isScanning || libraryRoots.length === 0}
                  className="win9x-button text-[0.8125rem] font-bold"
                >
                  {isScanning ? "Scanning..." : "Scan All Enabled"}
                </button>
              </div>

              {libraryRoots.length === 0 ? (
                <div className="p-4 text-center text-gray-600 win9x-inset">
                  No media library folders added yet. Add a directory above or test with the mock generator.
                </div>
              ) : (
                <div className="space-y-1.5">
                  {libraryRoots.map((root) => (
                    <div
                      key={root.id}
                      className="win9x-inset p-2 flex items-center justify-between gap-3 text-[0.875rem]"
                    >
                      <div className="flex items-center gap-2 flex-1 min-w-0">
                        <input
                          type="checkbox"
                          checked={root.enabled === 1}
                          onChange={(e) => handleToggle(root.id, e.target.checked)}
                          className="cursor-pointer"
                        />
                        <span className="font-bold text-[0.75rem] uppercase px-1.5 py-0.5 bg-gray-200 border border-gray-400">
                          {root.media_type}
                        </span>
                        <span className="font-mono text-[0.8125rem] truncate">
                          {root.path}
                        </span>
                      </div>

                      <div className="flex items-center gap-2 flex-shrink-0">
                        <span className="text-[0.8125rem] text-gray-500">
                          {root.last_scanned_at
                            ? `Last: ${root.last_scanned_at.slice(0, 16)}`
                            : "Never scanned"}
                        </span>
                        <button
                          onClick={() => triggerScan(root.id)}
                          disabled={isScanning}
                          className="win9x-button text-[0.8125rem]"
                        >
                          Scan
                        </button>
                        <button
                          onClick={() => handleRemove(root.id)}
                          className="win9x-button text-[0.8125rem] text-red-700 font-bold"
                        >
                          Remove
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Phase 4: Background File Watcher Control */}
            <div className="win9x-raised p-3 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-2 flex items-center justify-between">
                <span>Background Library Watcher (Phase 4)</span>
                <span
                  className={`text-[0.8125rem] font-bold ${
                    watcherStatus?.enabled && (watcherStatus?.watched_roots_count ?? 0) > 0
                      ? "text-[#005500]"
                      : "text-[#555555]"
                  }`}
                >
                  {watcherStatus?.enabled
                    ? `● ACTIVE (${watcherStatus.watched_roots_count} roots)`
                    : "○ DISABLED"}
                </span>
              </div>

              <div className="space-y-2 text-[0.875rem]">
                <label className="flex items-start gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={watcherStatus?.enabled ?? true}
                    onChange={(e) => setWatcherEnabled(e.target.checked)}
                    className="mt-1 cursor-pointer"
                  />
                  <div>
                    <div className="font-bold">
                      Automatically monitor library folders for file additions, moves, and deletions
                    </div>
                    <div className="text-[0.8125rem] text-gray-700 mt-0.5">
                      Uses debounced background filesystem events (`notify` crate) to incrementally discover new movies, episodes, and sidecar subtitles without manual rescans.
                    </div>
                  </div>
                </label>

                {watcherStatus?.enabled && watcherStatus.watched_paths.length > 0 && (
                  <div className="mt-2 win9x-inset p-2 bg-[#dfdfdf] space-y-1">
                    <div className="text-[0.8125rem] font-bold text-gray-700">Currently Monitored Paths:</div>
                    {watcherStatus.watched_paths.map((p, idx) => (
                      <div key={idx} className="font-mono text-[0.8125rem] text-gray-800 truncate">
                        ✓ {p}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Display & Scaling Tab */}
        {activeTab === "display" && (
          <div className="space-y-4 max-w-3xl text-[0.875rem]">
            {/* UI Scaling Configuration */}
            <div className="win9x-raised p-3 bg-[#c0c0c0] space-y-3">
              <div className="font-bold border-b border-[#808080] pb-1 flex justify-between items-center">
                <span>UI Display Scaling & Native Zoom</span>
                <span className="text-[0.8125rem] font-bold text-blue-900">
                  Current Scale: {Math.round(uiScale * 100)}%
                </span>
              </div>

              <p className="text-gray-700 leading-relaxed">
                PeppyMediaOrganizer scales all typography and layouts proportionally in <code>rem</code> units. Adjust the UI scale factor below to match your monitor resolution and personal readability preferences.
              </p>

              <div className="win9x-panel-inset p-3 space-y-3">
                <div className="flex items-center gap-3">
                  <label className="font-bold w-36">Preset UI Scale:</label>
                  <select
                    value={uiScale.toString()}
                    onChange={(e) => setUiScale(parseFloat(e.target.value))}
                    className="win9x-input flex-1 font-mono text-[0.875rem] cursor-pointer"
                  >
                    <option value="1">100% (Standard / Normal)</option>
                    <option value="1.1">110% (Slightly Larger)</option>
                    <option value="1.25">125% (Ubuntu / Windows 1080p Recommended)</option>
                    <option value="1.5">150% (High DPI / 1440p)</option>
                    <option value="1.75">175% (Ultra High DPI)</option>
                    <option value="2">200% (4K Displays)</option>
                  </select>
                </div>

                <div className="flex items-center gap-2 pt-1">
                  <button
                    type="button"
                    onClick={() => setUiScale(uiScale - 0.1)}
                    className="win9x-button font-bold text-[0.8125rem] !px-3"
                    title="Decrease zoom factor (Ctrl+-)"
                  >
                    🔍 Zoom Out (Ctrl+-)
                  </button>
                  <button
                    type="button"
                    onClick={() => setUiScale(uiScale + 0.1)}
                    className="win9x-button font-bold text-[0.8125rem] !px-3"
                    title="Increase zoom factor (Ctrl+=)"
                  >
                    🔎 Zoom In (Ctrl+=)
                  </button>
                  <button
                    type="button"
                    onClick={() => setUiScale(1.0)}
                    className="win9x-button font-bold text-[0.8125rem] !px-3 text-blue-900"
                    title="Reset zoom to 100% (Ctrl+0)"
                  >
                    ⟲ Reset (100%)
                  </button>
                </div>
              </div>
            </div>

            {/* System Scaling Diagnostics */}
            <div className="win9x-raised p-3 bg-[#c0c0c0] space-y-2">
              <div className="font-bold border-b border-[#808080] pb-1">
                Cross-OS Display Diagnostics
              </div>
              <div className="grid grid-cols-3 gap-2 text-[0.8125rem]">
                <div className="win9x-panel-inset p-2">
                  <div className="text-gray-600 font-bold">OS Text Scale:</div>
                  <div className="font-mono text-[0.9375rem] font-bold text-blue-900 mt-1">
                    {systemTextScale.toFixed(2)}x
                  </div>
                  <div className="text-[0.75rem] text-gray-500 mt-0.5">
                    (GDK_DPI_SCALE / GNOME)
                  </div>
                </div>

                <div className="win9x-panel-inset p-2">
                  <div className="text-gray-600 font-bold">Device Pixel Ratio:</div>
                  <div className="font-mono text-[0.9375rem] font-bold text-green-900 mt-1">
                    {typeof window !== "undefined" ? window.devicePixelRatio.toFixed(2) : "1.00"}x
                  </div>
                  <div className="text-[0.75rem] text-gray-500 mt-0.5">
                    (Webview DPI factor)
                  </div>
                </div>

                <div className="win9x-panel-inset p-2">
                  <div className="text-gray-600 font-bold">Applied Zoom:</div>
                  <div className="font-mono text-[0.9375rem] font-bold text-purple-900 mt-1">
                    {Math.round(uiScale * 100)}%
                  </div>
                  <div className="text-[0.75rem] text-gray-500 mt-0.5">
                    (Tauri set_zoom factor)
                  </div>
                </div>
              </div>
            </div>

            {/* Cross-Platform Details */}
            <div className="win9x-inset p-3 bg-white text-[0.8125rem] space-y-1.5 leading-relaxed">
              <div className="font-bold text-black border-b pb-1">
                Display Compatibility Notes
              </div>
              <p>
                <strong>Windows & macOS:</strong> WebView2 and WKWebView automatically respect OS fractional scaling (125%, 150%, 200%).
              </p>
              <p>
                <strong>Linux (WebKitGTK):</strong> Automatically inspects <code>GDK_DPI_SCALE</code>, <code>QT_SCALE_FACTOR</code>, and GNOME <code>text-scaling-factor</code> so Ubuntu fractional scaling applies crisp fonts without blurring.
              </p>
              <p>
                <strong>Font Stack:</strong> Primary open-source <code>W95FA</code> (SIL Open Font License) with seamless fallback to <code>Tahoma</code>, <code>Segoe UI</code>, <code>Ubuntu</code>, and <code>sans-serif</code>.
              </p>
            </div>
          </div>
        )}
        {activeTab === "providers" && (
          <div className="space-y-4 max-w-3xl text-[11px]">
            <form onSubmit={handleSaveProviders} className="win9x-raised p-3 bg-[#c0c0c0] space-y-3">
              <div className="font-bold border-b border-[#808080] pb-1">
                Metadata Provider API Credentials
              </div>
              <p className="text-gray-600">
                Configure your personal API keys for online providers. IMDb works completely offline without any API key.
              </p>

              {/* TMDb */}
              <div className="win9x-panel-inset p-2 space-y-1">
                <div className="flex justify-between items-center font-bold">
                  <span>TheMovieDB (TMDb)</span>
                  <span className="text-[10px] text-blue-900">Posters, Fanart, Overviews</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-24">API Key (v3):</span>
                  <input
                    type="password"
                    value={tmdbKey}
                    onChange={(e) => setTmdbKey(e.target.value)}
                    placeholder="Enter TMDb API key..."
                    className="win9x-input flex-1 font-mono"
                  />
                </div>
              </div>

              {/* TheTVDB */}
              <div className="win9x-panel-inset p-2 space-y-1">
                <div className="flex justify-between items-center font-bold">
                  <span>TheTVDB (v4)</span>
                  <span className="text-[10px] text-blue-900">Episode Ordering & Seasons</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-24">Project API Key:</span>
                  <input
                    type="password"
                    value={tvdbKey}
                    onChange={(e) => setTvdbKey(e.target.value)}
                    placeholder="Enter TVDb v4 key..."
                    className="win9x-input flex-1 font-mono"
                  />
                </div>
                <div className="flex items-center gap-2 pt-1">
                  <span className="w-24">User PIN (Optional):</span>
                  <input
                    type="password"
                    value={tvdbPin}
                    onChange={(e) => setTvdbPin(e.target.value)}
                    placeholder="User subscriber pin if applicable"
                    className="win9x-input flex-1 font-mono"
                  />
                </div>
              </div>

              {/* OMDb */}
              <div className="win9x-panel-inset p-2 space-y-1">
                <div className="flex justify-between items-center font-bold">
                  <span>OMDb API</span>
                  <span className="text-[10px] text-blue-900">Rotten Tomatoes, Metascore, Ratings</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="w-24">API Key:</span>
                  <input
                    type="password"
                    value={omdbKey}
                    onChange={(e) => setOmdbKey(e.target.value)}
                    placeholder="Enter OMDb API key..."
                    className="win9x-input flex-1 font-mono"
                  />
                </div>
              </div>

              <div className="flex items-center justify-between pt-2">
                <span className="text-blue-900 font-bold">{providersSavedMsg}</span>
                <button type="submit" className="win9x-button font-bold text-green-900 !px-4">
                  Save API Keys
                </button>
              </div>
            </form>
          </div>
        )}

        {/* Phase 2: IMDb Local Dataset Tab */}
        {activeTab === "imdb" && (
          <div className="space-y-4 max-w-3xl text-[11px]">
            <div className="win9x-raised p-3 bg-[#c0c0c0] space-y-3">
              <div className="font-bold border-b border-[#808080] pb-1 flex justify-between items-center">
                <span>Free IMDb Offline Dataset Engine</span>
                <span className="text-[10px] text-green-800 font-bold">Zero API Cost · Instant Search</span>
              </div>

              <p className="text-gray-700 leading-relaxed">
                PeppyMediaOrganizer can index IMDb's non-commercial TSV dumps (<code>title.basics.tsv.gz</code>, <code>title.ratings.tsv.gz</code>, <code>title.episode.tsv.gz</code>) into a dedicated SQLite cache. This allows instant title/year and <code>tt-ID</code> lookup without requiring an API.
              </p>

              {/* Local Folder Ingestion */}
              <div className="win9x-panel-inset p-2.5 space-y-2">
                <div className="font-bold">Index from Local Files (No Download Required)</div>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={imdbFolderPath}
                    onChange={(e) => setImdbFolderPath(e.target.value)}
                    className="win9x-input flex-1 font-mono text-[10px]"
                  />
                  <button onClick={handlePickImdbFolder} className="win9x-button">
                    Browse...
                  </button>
                  <button
                    onClick={handleIndexLocalImdb}
                    disabled={isIndexingImdb}
                    className="win9x-button font-bold text-blue-900 !px-3"
                  >
                    {isIndexingImdb ? "Indexing..." : "▶ Ingest & Index Local Files"}
                  </button>
                </div>
                {imdbStatusText && (
                  <div className="text-[10px] font-mono text-blue-900 bg-white p-1.5 win9x-inset">
                    {imdbStatusText}
                  </div>
                )}
              </div>

              {/* Current Local Database Status */}
              <div className="win9x-raised p-2 bg-[#dfdfdf] space-y-1">
                <div className="font-bold mb-1">Local Database Status:</div>
                <div className="grid grid-cols-3 gap-2">
                  {syncStatus.map((s) => (
                    <div key={s.dataset_name} className="win9x-panel-inset p-1.5 text-[10px]">
                      <div className="font-bold">{s.dataset_name}</div>
                      <div>Records: {s.record_count.toLocaleString()}</div>
                      <div className="text-gray-500">
                        {s.last_synced_at ? s.last_synced_at.slice(0, 16) : "Not indexed"}
                      </div>
                    </div>
                  ))}
                  {syncStatus.length === 0 && (
                    <div className="col-span-3 text-center text-gray-500 p-2 win9x-inset">
                      No datasets indexed yet. Click "Ingest & Index Local Files" above.
                    </div>
                  )}
                </div>
              </div>

              {/* Upstream Check */}
              <div className="win9x-panel-inset p-2 space-y-2">
                <div className="flex justify-between items-center">
                  <span className="font-bold">Upstream Daily Dataset Check (datasets.imdbws.com)</span>
                  <button
                    onClick={handleCheckUpstream}
                    disabled={isCheckingUpdates}
                    className="win9x-button font-bold text-xs"
                  >
                    {isCheckingUpdates ? "Checking..." : "Check for Updates"}
                  </button>
                </div>

                {updateInfos.length > 0 && (
                  <div className="space-y-1">
                    {updateInfos.map((u) => (
                      <div key={u.dataset_name} className="flex justify-between items-center win9x-inset p-1 text-[10px]">
                        <span className="font-bold font-mono">{u.dataset_name}</span>
                        <span>Upstream: {u.upstream_last_modified.slice(0, 16)}</span>
                        <span className={u.update_available ? "text-amber-700 font-bold" : "text-green-700 font-bold"}>
                          {u.update_available ? "Update Available" : "✓ Up to date"}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Phase 2: Field Priorities Tab */}
        {activeTab === "priorities" && (
          <div className="space-y-4 max-w-3xl text-[11px]">
            <div className="win9x-raised p-3 bg-[#c0c0c0] space-y-3">
              <div className="font-bold border-b border-[#808080] pb-1">
                Metadata Field Priority Matrix
              </div>
              <p className="text-gray-700">
                When scraping media items, PeppyMediaOrganizer queries enabled providers in parallel and evaluates field data according to your custom priority rules.
              </p>

              <div className="space-y-2">
                {Object.entries(priorities).map(([field, order]) => (
                  <div key={field} className="win9x-panel-inset p-2 flex items-center justify-between">
                    <span className="font-bold capitalize w-32">{field}:</span>
                    <div className="flex items-center gap-1 font-mono text-[10px]">
                      {order.map((p, idx) => (
                        <React.Fragment key={p}>
                          <span className="px-1.5 py-0.5 bg-white border border-gray-400 uppercase font-bold text-blue-900">
                            {p}
                          </span>
                          {idx < order.length - 1 && <span className="text-gray-500">→</span>}
                        </React.Fragment>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === "mediainfo" && (
          <div className="space-y-4 max-w-3xl">
            {/* Binary Configuration Box */}
            <div className="win9x-raised p-2.5 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-2">
                ffprobe Binary Configuration
              </div>
              <p className="text-xs text-gray-700 mb-3 leading-relaxed">
                PeppyMediaOrganizer uses <code>ffprobe</code> (from the FFmpeg suite) to inspect media files, extracting video codecs (H.264, HEVC, AV1), dynamic range (HDR10, Dolby Vision, SDR), audio formats, and duration.
              </p>

              <form onSubmit={handleSaveFfprobe} className="space-y-3">
                <div>
                  <label className="block text-xs font-bold mb-1">
                    ffprobe Executable Path:
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={ffprobePath}
                      onChange={(e) => setFfprobePath(e.target.value)}
                      placeholder="ffprobe (auto-detected from system PATH)"
                      className="win9x-input flex-1 font-mono text-xs"
                    />
                    <button
                      type="button"
                      onClick={handlePickFfprobeBinary}
                      className="win9x-button text-xs"
                    >
                      Browse...
                    </button>
                    <button
                      type="button"
                      onClick={loadFfprobe}
                      className="win9x-button text-xs"
                      title="Re-check system PATH for ffprobe"
                    >
                      Auto-Detect
                    </button>
                  </div>
                  <div className="text-[10px] text-gray-600 mt-1">
                    Leave blank or set to &quot;ffprobe&quot; to auto-detect from system PATH.
                  </div>
                  <div className="flex items-center gap-2 mt-1.5 text-[11px]">
                    <span className="font-bold">Status:</span>
                    {ffprobeDetectedPath ? (
                      <span className="text-green-800 font-bold bg-green-50 px-1.5 py-0.5 border border-green-300 font-mono">
                        ✓ Available ({ffprobeDetectedPath})
                      </span>
                    ) : (
                      <span className="text-red-700 font-bold bg-red-50 px-1.5 py-0.5 border border-red-300">
                        ✗ Not found (Install ffmpeg/ffprobe or specify executable path above)
                      </span>
                    )}
                  </div>
                </div>

                <div className="flex items-center gap-3 pt-1">
                  <button type="submit" className="win9x-button font-bold text-xs">
                    Save Path
                  </button>
                  {ffprobeSavedMsg && (
                    <span className="text-xs text-green-800 font-bold">
                      {ffprobeSavedMsg}
                    </span>
                  )}
                </div>
              </form>
            </div>

            {/* Background Worker Controls */}
            <div className="win9x-raised p-2.5 bg-[#c0c0c0]">
              <div className="font-bold border-b border-[#808080] pb-1 mb-2">
                Background Stream Analysis Queue
              </div>
              <p className="text-xs text-gray-700 mb-3 leading-relaxed">
                Initial library scans extract filenames and NFOs immediately so your library remains responsive. Technical audio/video streams are probed incrementally in a non-blocking queue.
              </p>

              <div className="flex items-center gap-3">
                <button
                  type="button"
                  onClick={handleRunBatchProbe}
                  disabled={isProbingBatch}
                  className="win9x-button font-bold text-xs text-blue-900"
                >
                  {isProbingBatch ? "⏳ Probing Media Files..." : "▶ Probe Next Batch (25 Files)"}
                </button>
                {probeBatchResult && (
                  <span className="text-xs font-bold text-gray-800 font-mono">
                    {probeBatchResult}
                  </span>
                )}
              </div>
            </div>

            {/* Technical Format Support Reference */}
            <div className="win9x-inset p-3 bg-white text-[11px] space-y-1.5">
              <div className="font-bold text-black border-b pb-1">
                Supported Media Stream Formats
              </div>
              <div className="grid grid-cols-2 gap-2 text-[10px]">
                <div>
                  <span className="font-bold text-blue-900">Video Codecs:</span>
                  <div>HEVC / H.265, AVC / H.264, AV1, VP9, MPEG-2</div>
                </div>
                <div>
                  <span className="font-bold text-purple-900">Dynamic Range:</span>
                  <div>Dolby Vision (Profile 5/7/8), HDR10, HLG, SDR</div>
                </div>
                <div>
                  <span className="font-bold text-amber-900">Audio Formats:</span>
                  <div>Dolby TrueHD, DTS-HD MA, EAC3 / DD+, AAC, FLAC</div>
                </div>
                <div>
                  <span className="font-bold text-green-900">Containers:</span>
                  <div>MKV, MP4, M4V, TS, AVI, MOV</div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === "rules" && (
          <div className="space-y-3 max-w-3xl text-[11px] win9x-inset p-3 bg-white leading-relaxed">
            <h4 className="font-bold text-sm text-blue-900 border-b pb-1">
              Jellyfin Filesystem Standards (Implemented)
            </h4>
            <div className="space-y-2">
              <div className="font-bold text-black">1. Provider ID Tag Format</div>
              <ul className="list-disc pl-5 space-y-0.5 font-mono text-[10px]">
                <li>IMDb: [imdbid-tt1234567] (always includes 'tt' prefix)</li>
                <li>TMDb: [tmdbid-12345]</li>
                <li>TVDb: [tvdbid-12345]</li>
                <li>Plex {'{curly brace}'} conventions are strictly forbidden.</li>
              </ul>

              <div className="font-bold text-black pt-1">2. Movie Organization</div>
              <p>Dedicated folder: <code>Movie Name (Year) [imdbid-tt...]</code></p>
              <p>Multi-version files: <code>Movie (Year) [id] - 2160p Final Cut.mkv</code></p>

              <div className="font-bold text-black pt-1">3. TV Organization</div>
              <p>Series folder: <code>Series Name (Year) [tvdbid-...]</code></p>
              <p>Season folders: <code>Season 01</code>, <code>Season 02</code> (Season 00 for specials)</p>
              <p>Episodes: <code>Series Name S01E01 - Title.mkv</code> (Multi-ep: <code>S01E01-E02</code>)</p>
            </div>
          </div>
        )}

        {activeTab === "about" && (
          <div className="space-y-3 max-w-xl win9x-inset p-4 bg-white">
            <div className="flex items-center gap-3 border-b pb-2">
              <div className="w-10 h-10 bg-[#000080] text-white flex items-center justify-center font-bold text-lg win9x-raised">
                PM
              </div>
              <div>
                <h3 className="font-bold text-base text-black">PeppyMediaOrganizer</h3>
                <div className="text-xs text-gray-500">Version 0.2.0 (Phase 2)</div>
              </div>
            </div>

            <p className="text-xs leading-relaxed text-gray-800">
              A fast, cross-platform desktop media manager specifically tailored for Jellyfin libraries.
              Features a multi-threaded parallel directory scanner in Rust, embedded SQLite in WAL mode, offline IMDb dataset search, TV episode remapping (Aired, DVD, Streaming, Absolute), and strict Jellyfin NFO XML generation.
            </p>

            <div className="text-[10px] text-gray-500 pt-2 border-t">
              Open source under MIT License.
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
