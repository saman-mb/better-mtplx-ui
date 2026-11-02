import { useEffect, useState } from "react";
import { InstanceGrid } from "./components/InstanceGrid";
import { LibraryView } from "./components/LibraryView";
import { SettingsView } from "./components/SettingsView";
import { MemoryBar } from "./components/MemoryBar";
import { useTick } from "./hooks/useTick";
import {
  getSettings,
  pullModel,
  startInstance,
  stopInstance,
  suggestPort,
} from "./lib/ipc";
import type { Settings, ViewName } from "./lib/types";
import { formatBytes } from "./lib/format";

export default function App() {
  const tick = useTick();
  const [view, setView] = useState<ViewName>("instances");
  const [settings, setSettingsState] = useState<Settings | null>(null);
  const [downloading, setDownloading] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<{
    kind: "error" | "info";
    text: string;
  } | null>(null);

  useEffect(() => {
    getSettings()
      .then(setSettingsState)
      .catch(() => setSettingsState(null));
  }, []);

  const flash = (kind: "error" | "info", text: string) => {
    setNotice({ kind, text });
    setTimeout(() => setNotice(null), 6000);
  };

  const launch = async (model: string) => {
    if (busy) return;
    setBusy(true);
    try {
      const port = await suggestPort();
      const extraArgs = buildServeArgs(settings);
      await startInstance(model, port, extraArgs);
      setView("instances");
      flash("info", `Started ${model} on port ${port}.`);
    } catch (err) {
      flash("error", String(err));
    } finally {
      setBusy(false);
    }
  };

  const stop = async (id: string) => {
    try {
      await stopInstance(id);
    } catch (err) {
      flash("error", String(err));
    }
  };

  const pull = async (repo: string) => {
    setDownloading((prev) => new Set(prev).add(repo));
    try {
      await pullModel(repo);
      flash("info", `Downloading ${repo}.`);
    } catch (err) {
      flash("error", String(err));
    } finally {
      // Keep the row in the "downloading" state until the pack validates.
      setTimeout(() => {
        setDownloading((prev) => {
          const next = new Set(prev);
          next.delete(repo);
          return next;
        });
      }, 4000);
    }
  };

  const liveCount = tick.instances.filter((i) =>
    ["starting", "loading", "ready", "stopping"].includes(i.status),
  ).length;

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-ink-900">
      <aside className="flex w-60 shrink-0 flex-col border-r border-ink-800 bg-ink-850">
        <div className="px-5 pb-4 pt-5">
          <div className="flex items-center gap-2">
            <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-accent-500 text-sm font-bold text-ink-950">
              ⬢
            </div>
            <div className="min-w-0">
              <h1 className="truncate text-xs font-semibold text-slate-100">
                Better MTPLX UI
              </h1>
              <p className="text-[10px] text-slate-500">parallel host</p>
            </div>
          </div>
        </div>

        <nav className="flex flex-col gap-0.5 px-3">
          <NavItem
            active={view === "instances"}
            onClick={() => setView("instances")}
            label="Instances"
            badge={liveCount}
          />
          <NavItem
            active={view === "library"}
            onClick={() => setView("library")}
            label="Library"
            badge={downloading.size}
          />
          <NavItem
            active={view === "settings"}
            onClick={() => setView("settings")}
            label="Settings"
          />
        </nav>

        <div className="mt-auto px-5 pb-5">
          <MemoryBar
            usedBytes={tick.memory.usedBytes}
            totalBytes={tick.memory.totalBytes}
            label="Unified memory"
          />
          <p className="mt-2 font-mono text-[10px] text-slate-500">
            {formatBytes(tick.memory.availableBytes)} available
          </p>
        </div>
      </aside>

      <main className="relative flex-1 overflow-hidden">
        {notice && (
          <div
            className={`absolute right-6 top-5 z-10 max-w-md rounded-lg px-4 py-2.5 text-xs shadow-lg ring-1 ${
              notice.kind === "error"
                ? "bg-rose-500/15 text-rose-200 ring-rose-500/30"
                : "bg-accent-500/15 text-accent-300 ring-accent-500/30"
            }`}
          >
            {notice.text}
          </div>
        )}

        {view === "instances" && (
          <InstanceGrid
            instances={tick.instances}
            totalMemoryBytes={tick.memory.totalBytes}
            onStop={(id) => void stop(id)}
            busy={busy}
            onOpenLibrary={() => setView("library")}
          />
        )}

        {view === "library" && (
          <LibraryView
            onLaunch={(model) => void launch(model)}
            downloading={downloading}
            onPull={(repo) => void pull(repo)}
          />
        )}

        {view === "settings" && <SettingsView />}
      </main>
    </div>
  );
}

/** Build the extra `mtplx serve` flags from the user's settings. */
function buildServeArgs(settings: Settings | null): string[] {
  if (!settings) return [];
  const args: string[] = [];

  if (settings.defaultProfile) {
    args.push("--profile", settings.defaultProfile);
  }
  if (settings.schedulerMode) {
    args.push("--scheduler-mode", settings.schedulerMode);
  }
  if (settings.contextWindow) {
    args.push("--context-window", String(settings.contextWindow));
  }
  if (settings.apiKey) {
    args.push("--api-key", settings.apiKey);
  }
  args.push(...settings.extraArgs);

  return args;
}

function NavItem({
  active,
  onClick,
  label,
  badge,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
  badge?: number;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center justify-between rounded-lg px-3 py-2 text-xs font-medium transition-colors ${
        active
          ? "bg-ink-700 text-slate-100"
          : "text-slate-400 hover:bg-ink-800 hover:text-slate-200"
      }`}
    >
      <span>{label}</span>
      {badge ? (
        <span className="rounded-full bg-accent-500/20 px-1.5 py-0.5 text-[10px] font-semibold text-accent-300">
          {badge}
        </span>
      ) : null}
    </button>
  );
}
