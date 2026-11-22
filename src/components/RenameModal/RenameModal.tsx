import React, { useState } from "react";
import { useMediaStore } from "../../hooks/useMediaStore";
import { api } from "../../lib/tauri";
import type { RenameAction, RenamePlan } from "../../lib/types";

interface RenameModalProps {
  isOpen: boolean;
  plan: RenamePlan | null;
  isLoading: boolean;
  onClose: () => void;
  onExecuted?: () => void;
}

export const RenameModal: React.FC<RenameModalProps> = ({
  isOpen,
  plan,
  isLoading,
  onClose,
  onExecuted,
}) => {
  const loadMovies = useMediaStore((s) => s.loadMovies);
  const loadTvShows = useMediaStore((s) => s.loadTvShows);

  // Selected action IDs for execution
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => {
    if (!plan) return new Set();
    return new Set(
      plan.actions
        .filter((a) => a.status === "will_rename")
        .map((a) => a.id)
    );
  });

  const [filter, setFilter] = useState<"all" | "will_rename" | "conflict" | "skipped">("all");
  const [isExecuting, setIsExecuting] = useState(false);
  const [statusMsg, setStatusMsg] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  // Sync selectedIds whenever a new plan arrives
  React.useEffect(() => {
    if (plan) {
      setSelectedIds(
        new Set(
          plan.actions
            .filter((a) => a.status === "will_rename")
            .map((a) => a.id)
        )
      );
      setStatusMsg("");
      setErrorMsg("");
    }
  }, [plan]);

  if (!isOpen) return null;

  const toggleAction = (id: string) => {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setSelectedIds(next);
  };

  const handleSelectAll = () => {
    if (!plan) return;
    const actionable = plan.actions
      .filter((a) => a.status === "will_rename")
      .map((a) => a.id);
    setSelectedIds(new Set(actionable));
  };

  const handleDeselectAll = () => {
    setSelectedIds(new Set());
  };

  const handleExecute = async () => {
    if (!plan || selectedIds.size === 0) return;
    try {
      setIsExecuting(true);
      setErrorMsg("");
      setStatusMsg(`Executing rename on ${selectedIds.size} items...`);

      const res = await api.executeRenamePlan(plan, Array.from(selectedIds));

      if (res.failed_count > 0) {
        setErrorMsg(`Executed ${res.executed_count} items with ${res.failed_count} errors: ${res.errors.join("; ")}`);
      } else {
        setStatusMsg(`✅ Successfully renamed ${res.executed_count} items conforming to Jellyfin standards!`);
      }

      // Refresh frontend stores
      await Promise.all([loadMovies(), loadTvShows()]);

      if (onExecuted) onExecuted();

      if (res.failed_count === 0) {
        setTimeout(() => onClose(), 1200);
      }
    } catch (err) {
      setErrorMsg(`Rename error: ${err}`);
    } finally {
      setIsExecuting(false);
    }
  };

  const filteredActions = plan?.actions.filter((a) => {
    if (filter === "all") return true;
    return a.status === filter;
  }) || [];

  const getStatusBadge = (status: RenameAction["status"]) => {
    switch (status) {
      case "will_rename":
        return <span className="text-green-800 font-bold bg-green-100 px-1 border border-green-600">✓ RENAME</span>;
      case "conflict":
        return <span className="text-red-800 font-bold bg-red-100 px-1 border border-red-600">⚠ CONFLICT</span>;
      case "skipped":
        return <span className="text-gray-600 bg-gray-100 px-1 border border-gray-400">⊘ SKIP</span>;
    }
  };

  const getItemTypeBadge = (itemType: RenameAction["item_type"]) => {
    switch (itemType) {
      case "movie_folder":
      case "tv_show_folder":
      case "season_folder":
        return <span className="text-amber-800 font-bold">[Folder]</span>;
      case "movie_video":
      case "episode_video":
        return <span className="text-blue-900 font-bold">[Video]</span>;
      case "subtitle":
        return <span className="text-purple-900">[Sub]</span>;
      case "nfo":
        return <span className="text-emerald-900">[NFO]</span>;
      case "artwork":
        return <span className="text-cyan-900">[Art]</span>;
      default:
        return <span className="text-gray-700">[Extra]</span>;
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
      <div className="w-[1100px] max-w-[95vw] max-h-[90vh] win9x-raised flex flex-col bg-[#c0c0c0] shadow-2xl overflow-hidden select-none">
        {/* Title Bar */}
        <div className="win9x-titlebar flex items-center justify-between px-2">
          <span>Jellyfin Standards Rename Preview (Dry-Run)</span>
          <button
            onClick={onClose}
            className="win9x-button !min-h-[1.5rem] !px-1.5 text-[0.8125rem] font-bold"
          >
            ✕
          </button>
        </div>

        {/* Content Area */}
        <div className="flex-1 flex flex-col p-2.5 gap-2 overflow-hidden text-[0.875rem]">
          {/* Summary / Controls Strip */}
          <div className="win9x-raised p-2 flex items-center justify-between bg-[#dfdfdf] flex-wrap gap-2">
            <div className="flex items-center gap-3">
              <span className="font-bold">Plan Summary:</span>
              <span>Total: <b>{plan?.total_count || 0}</b></span>
              <span className="text-green-800 font-bold bg-green-50 px-1.5 py-0.5 border border-green-300">
                Will Rename: {plan?.will_rename_count || 0}
              </span>
              <span className="text-red-800 font-bold bg-red-50 px-1.5 py-0.5 border border-red-300">
                Conflicts: {plan?.conflict_count || 0}
              </span>
              <span className="text-gray-700 bg-gray-50 px-1.5 py-0.5 border border-gray-300">
                Skipped: {plan?.skipped_count || 0}
              </span>
            </div>

            <div className="flex items-center gap-2">
              <span className="font-bold">Filter:</span>
              <select
                value={filter}
                onChange={(e) => setFilter(e.target.value as any)}
                className="win9x-input text-[0.8125rem]"
              >
                <option value="all">All Items ({plan?.total_count || 0})</option>
                <option value="will_rename">Will Rename ({plan?.will_rename_count || 0})</option>
                <option value="conflict">Conflicts ({plan?.conflict_count || 0})</option>
                <option value="skipped">Skipped ({plan?.skipped_count || 0})</option>
              </select>

              <button onClick={handleSelectAll} className="win9x-button text-[0.8125rem]">
                Select All
              </button>
              <button onClick={handleDeselectAll} className="win9x-button text-[0.8125rem]">
                Deselect All
              </button>
            </div>
          </div>

          {/* Table Container */}
          <div className="flex-1 win9x-inset overflow-y-auto bg-white">
            {isLoading ? (
              <div className="p-8 text-center text-gray-500 font-bold">
                Analyzing library and generating dry-run rename plan...
              </div>
            ) : filteredActions.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                No items match current filter.
              </div>
            ) : (
              <table className="w-full border-collapse text-[0.8125rem]">
                <thead className="sticky top-0 bg-[#c0c0c0] win9x-raised z-10">
                  <tr className="border-b border-[#808080] text-left">
                    <th className="p-1.5 w-8 text-center">✓</th>
                    <th className="p-1.5 w-28">Status</th>
                    <th className="p-1.5 w-20">Type</th>
                    <th className="p-1.5 w-[40%]">Old Path (Source)</th>
                    <th className="p-1.5 w-6 text-center">→</th>
                    <th className="p-1.5 w-[40%]">New Path (Jellyfin Target)</th>
                    <th className="p-1.5">Notes</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredActions.map((action) => {
                    const isSelected = selectedIds.has(action.id);
                    const isConflict = action.status === "conflict";
                    const isSkipped = action.status === "skipped";

                    return (
                      <tr
                        key={action.id}
                        className={`border-b border-[#f0f0f0] hover:bg-blue-50 ${
                          isConflict ? "bg-red-50/50" : isSkipped ? "opacity-75 bg-gray-50/50" : ""
                        }`}
                      >
                        <td className="p-1.5 text-center">
                          <input
                            type="checkbox"
                            checked={isSelected}
                            disabled={isConflict || isSkipped}
                            onChange={() => toggleAction(action.id)}
                            className="cursor-pointer"
                          />
                        </td>
                        <td className="p-1.5 whitespace-nowrap">{getStatusBadge(action.status)}</td>
                        <td className="p-1.5 whitespace-nowrap">{getItemTypeBadge(action.item_type)}</td>
                        <td className="p-1.5 font-mono text-[0.8125rem] break-all max-w-[280px]">
                          {action.old_path}
                        </td>
                        <td className="p-1.5 text-center font-bold text-blue-900">→</td>
                        <td className="p-1.5 font-mono text-[0.8125rem] font-bold text-blue-950 break-all max-w-[280px]">
                          {action.new_path}
                        </td>
                        <td className="p-1.5 text-[0.75rem] text-gray-600 truncate max-w-[140px]">
                          {action.reason || ""}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            )}
          </div>

          {/* Status & Error Messages */}
          {errorMsg && (
            <div className="p-2 win9x-inset bg-red-100 text-red-900 font-bold text-[0.8125rem]">
              ⚠ {errorMsg}
            </div>
          )}
          {statusMsg && (
            <div className="p-2 win9x-inset bg-green-100 text-green-900 font-bold text-[0.8125rem]">
              {statusMsg}
            </div>
          )}

          {/* Footer Controls */}
          <div className="win9x-raised p-2 flex items-center justify-between bg-[#c0c0c0]">
            <div className="text-[0.8125rem] text-gray-700">
              Selected for renaming: <b>{selectedIds.size}</b> of {plan?.will_rename_count || 0} actionable files
            </div>

            <div className="flex items-center gap-2">
              <button onClick={onClose} disabled={isExecuting} className="win9x-button text-[0.8125rem]">
                Cancel
              </button>
              <button
                onClick={handleExecute}
                disabled={isExecuting || selectedIds.size === 0}
                className="win9x-button font-bold text-green-900 text-[0.8125rem] !px-4"
              >
                {isExecuting ? "Renaming..." : `✓ Execute Rename (${selectedIds.size} Items)`}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
