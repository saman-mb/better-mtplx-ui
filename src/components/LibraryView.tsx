import { useCallback, useEffect, useState } from "react";
import { discoverModels, listModels } from "../lib/ipc";
import type { DiscoverCard, ModelPack } from "../lib/types";
import { formatBytes, shortRepo } from "../lib/format";

interface Props {
  /** Called with a repo id to spin up an instance. */
  onLaunch: (model: string) => void;
  /** Repos currently being downloaded. */
  downloading: Set<string>;
  onPull: (repo: string) => void;
}

type Tab = "installed" | "catalog";

/**
 * The model library: what is on disk, and what can be fetched.
 *
 * Two tabs, one job each — "run what I have" and "get what I want".
 */
export function LibraryView({ onLaunch, downloading, onPull }: Props) {
  const [tab, setTab] = useState<Tab>("installed");
  const [packs, setPacks] = useState<ModelPack[]>([]);
  const [catalog, setCatalog] = useState<DiscoverCard[]>([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refreshPacks = useCallback(async () => {
    try {
      const payload = await listModels();
      setPacks(payload.models);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  useEffect(() => {
    void refreshPacks();
    // Poll while downloads are in flight so progress shows up.
    if (downloading.size === 0) return;
    const timer = setInterval(() => void refreshPacks(), 2000);
    return () => clearInterval(timer);
  }, [downloading, refreshPacks]);

  useEffect(() => {
    if (tab !== "catalog" || catalog.length > 0) return;
    setLoading(true);
    discoverModels(undefined, 40)
      .then((cards) => setCatalog(cards))
      .catch((err) => setError(String(err)))
      .finally(() => setLoading(false));
  }, [tab, catalog.length]);

  const filtered = catalog.filter((card) =>
    card.repo.toLowerCase().includes(query.trim().toLowerCase()),
  );

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-1 border-b border-ink-800 px-6 pt-4">
        <TabButton
          active={tab === "installed"}
          onClick={() => setTab("installed")}
        >
          Installed
          <span className="ml-2 text-[11px] text-slate-500">
            {packs.length}
          </span>
        </TabButton>
        <TabButton active={tab === "catalog"} onClick={() => setTab("catalog")}>
          Catalog
        </TabButton>
      </div>

      {error && (
        <div className="mx-6 mt-4 rounded-lg bg-rose-500/10 px-3 py-2 text-xs text-rose-300">
          {error}
        </div>
      )}

      {tab === "installed" ? (
        <div className="flex-1 overflow-y-auto px-6 py-5">
          {packs.length === 0 ? (
            <p className="text-xs text-slate-500">
              Nothing cached yet. Switch to the Catalog tab to download a pack.
            </p>
          ) : (
            <div className="grid grid-cols-1 gap-3 lg:grid-cols-2 2xl:grid-cols-3">
              {packs.map((pack) => (
                <PackCard
                  key={pack.repo_id}
                  pack={pack}
                  onLaunch={() => onLaunch(pack.repo_id)}
                  downloading={downloading.has(pack.repo_id)}
                />
              ))}
            </div>
          )}
        </div>
      ) : (
        <div className="flex flex-1 flex-col overflow-y-auto px-6 py-5">
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Filter by name or owner…"
            className="mb-4 w-full max-w-sm rounded-lg border border-ink-700 bg-ink-850 px-3 py-2 text-xs text-slate-200 outline-none placeholder:text-slate-500 focus:border-accent-500"
          />
          {loading ? (
            <p className="text-xs text-slate-500">Searching Hugging Face…</p>
          ) : (
            <div className="grid grid-cols-1 gap-3 lg:grid-cols-2 2xl:grid-cols-3">
              {filtered.map((card) => (
                <div
                  key={card.repo}
                  className="flex flex-col rounded-xl border border-ink-700 bg-ink-800 p-4"
                >
                  <h3 className="truncate text-sm font-semibold text-slate-100">
                    {shortRepo(card.repo)}
                  </h3>
                  <p className="mt-0.5 truncate font-mono text-[11px] text-slate-500">
                    {card.repo}
                  </p>
                  <div className="mt-3 flex items-center gap-3 text-[11px] text-slate-500">
                    <span>{card.downloads.toLocaleString()} downloads</span>
                    {card.license && <span>· {card.license}</span>}
                  </div>
                  <div className="mt-4 flex items-center gap-2">
                    <button
                      onClick={() => onPull(card.repo)}
                      disabled={downloading.has(card.repo)}
                      className="rounded-md bg-accent-500 px-3 py-1.5 text-xs font-semibold text-ink-950 transition-colors hover:bg-accent-400 disabled:opacity-50"
                    >
                      {downloading.has(card.repo) ? "Downloading…" : "Download"}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`border-b-2 px-3 pb-2.5 text-xs font-medium transition-colors ${
        active
          ? "border-accent-400 text-slate-100"
          : "border-transparent text-slate-500 hover:text-slate-300"
      }`}
    >
      {children}
    </button>
  );
}

function PackCard({
  pack,
  onLaunch,
  downloading,
}: {
  pack: ModelPack;
  onLaunch: () => void;
  downloading: boolean;
}) {
  const complete =
    pack.validation?.contract_present === true &&
    (pack.validation?.missing_files.length ?? 0) === 0;

  return (
    <div className="flex flex-col rounded-xl border border-ink-700 bg-ink-800 p-4">
      <h3 className="truncate text-sm font-semibold text-slate-100">
        {shortRepo(pack.repo_id)}
      </h3>
      <p className="mt-0.5 truncate font-mono text-[11px] text-slate-500">
        {pack.repo_id}
      </p>

      <div className="mt-3 flex items-center gap-2 text-[11px]">
        <span className="text-slate-400">
          {pack.size_gb ? `${pack.size_gb.toFixed(1)} GB` : "size unknown"}
        </span>
        {complete ? (
          <span className="rounded-full bg-emerald-500/15 px-2 py-0.5 text-emerald-300 ring-1 ring-emerald-500/30">
            ready
          </span>
        ) : (
          <span className="rounded-full bg-amber-500/15 px-2 py-0.5 text-amber-300 ring-1 ring-amber-500/30">
            downloading
          </span>
        )}
      </div>

      {!complete && pack.validation?.missing_files.length ? (
        <p className="mt-2 text-[11px] leading-relaxed text-slate-500">
          Waiting on {pack.validation.missing_files.length} file(s).
        </p>
      ) : null}

      <div className="mt-4 flex items-center gap-2">
        <button
          onClick={onLaunch}
          disabled={!complete}
          className="rounded-md bg-accent-500 px-3 py-1.5 text-xs font-semibold text-ink-950 transition-colors hover:bg-accent-400 disabled:cursor-not-allowed disabled:bg-ink-700 disabled:text-slate-500"
        >
          {downloading ? "Downloading…" : "Run"}
        </button>
        {pack.recommended_profile && (
          <span className="text-[11px] text-slate-500">
            profile: {pack.recommended_profile}
          </span>
        )}
      </div>
    </div>
  );
}

/** Bytes helper re-export so the library view stays self-contained. */
export const formatPackSize = formatBytes;
