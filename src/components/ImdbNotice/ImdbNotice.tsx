import React from "react";
import { useMediaStore } from "../../hooks/useMediaStore";

export const ImdbNotice: React.FC = () => {
  const isImdbInstalled = useMediaStore((s) => s.isImdbInstalled);
  const isImdbSyncing = useMediaStore((s) => s.isImdbSyncing);
  const imdbSyncProgress = useMediaStore((s) => s.imdbSyncProgress);
  const imdbPromptDismissed = useMediaStore((s) => s.imdbPromptDismissed);
  const startImdbDownload = useMediaStore((s) => s.startImdbDownload);
  const dismissImdbPrompt = useMediaStore((s) => s.dismissImdbPrompt);

  // If installed and not currently syncing, don't show the banner
  if (isImdbInstalled === true && !isImdbSyncing) {
    return null;
  }

  // If dismissed and not syncing, don't show
  if (imdbPromptDismissed && !isImdbSyncing) {
    return null;
  }

  // If still determining initial install status (null), don't flash banner
  if (isImdbInstalled === null && !isImdbSyncing) {
    return null;
  }

  return (
    <div className="win9x-raised mx-1 mt-1 p-2 bg-[#dfdfdf] border border-[#808080] flex items-center justify-between gap-3 text-[0.8125rem] select-none shadow-sm flex-shrink-0">
      <div className="flex items-center gap-2 min-w-0">
        <span className="text-base flex-shrink-0">
          {isImdbSyncing ? "⏳" : "ℹ️"}
        </span>
        <div className="truncate">
          {isImdbSyncing ? (
            <div>
              <span className="font-bold text-blue-900">
                IMDb Background Ingestion Active:
              </span>{" "}
              <span className="font-mono text-gray-800">
                {imdbSyncProgress || "Downloading & indexing datasets..."}
              </span>
              <span className="text-[0.75rem] text-gray-500 ml-2">
                (Full library editing and scanning remain available)
              </span>
            </div>
          ) : (
            <div>
              <span className="font-bold text-black">
                IMDb Offline Dataset Not Installed:
              </span>{" "}
              <span className="text-gray-700">
                Download and index <code>title.basics</code>, <code>title.ratings</code>, and <code>title.episode</code> (~100MB compressed) for instant offline tt-ID matching.
              </span>
            </div>
          )}
        </div>
      </div>

      <div className="flex items-center gap-1.5 flex-shrink-0">
        {!isImdbSyncing && (
          <>
            <button
              onClick={() => startImdbDownload()}
              className="win9x-button font-bold text-blue-900 !px-3 !py-1 text-[0.8125rem]"
              title="Download datasets.imdbws.com archives into app_data_dir, stream into imdb.db, and delete temporary files immediately"
            >
              ▶ Download & Index in Background
            </button>
            <button
              onClick={() => dismissImdbPrompt()}
              className="win9x-button text-gray-700 !px-2.5 !py-1 text-[0.8125rem]"
              title="Dismiss this notice for the current session"
            >
              Dismiss
            </button>
          </>
        )}
      </div>
    </div>
  );
};
