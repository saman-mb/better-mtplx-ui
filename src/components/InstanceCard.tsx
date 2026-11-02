import { useState } from "react";
import type { InstanceView } from "../lib/types";
import { formatBytes, shortRepo } from "../lib/format";
import { StatusBadge } from "./StatusBadge";

interface Props {
  instance: InstanceView;
  totalMemoryBytes: number;
  onStop: (id: string) => void;
  busy: boolean;
}

/**
 * One running model instance.
 *
 * The card is the unit of the whole app: everything the user needs to know
 * about a model — which port answers, how much memory it holds, whether it is
 * healthy — lives here without opening anything.
 */
export function InstanceCard({
  instance,
  totalMemoryBytes,
  onStop,
  busy,
}: Props) {
  const [showLogs, setShowLogs] = useState(false);
  const isLive = ["starting", "loading", "ready", "stopping"].includes(
    instance.status,
  );

  return (
    <article
      className={`flex flex-col rounded-xl border bg-ink-800 p-4 transition-colors ${
        instance.status === "failed"
          ? "border-rose-500/40"
          : instance.status === "ready"
            ? "border-emerald-500/30"
            : "border-ink-700"
      }`}
    >
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h3
            className="truncate text-sm font-semibold text-slate-100"
            title={instance.model}
          >
            {shortRepo(instance.model)}
          </h3>
          <p className="mt-0.5 truncate font-mono text-[11px] text-slate-500">
            {instance.model}
          </p>
        </div>
        <StatusBadge status={instance.status} />
      </header>

      <div className="mt-4 grid grid-cols-2 gap-3">
        <div className="rounded-lg bg-ink-900/60 px-3 py-2">
          <div className="text-[10px] uppercase tracking-wider text-slate-500">
            Endpoint
          </div>
          <div className="mt-0.5 font-mono text-xs text-accent-400">
            127.0.0.1:{instance.port}
          </div>
        </div>
        <div className="rounded-lg bg-ink-900/60 px-3 py-2">
          <div className="text-[10px] uppercase tracking-wider text-slate-500">
            Throughput
          </div>
          <div className="mt-0.5 font-mono text-xs text-slate-200">
            {instance.tokensPerSecond
              ? `${instance.tokensPerSecond.toFixed(1)} tok/s`
              : "—"}
          </div>
        </div>
      </div>

      <div className="mt-4">
        <MemoryBarInline
          usedBytes={instance.weightsBytes ?? instance.memoryBytes}
          totalBytes={totalMemoryBytes}
        />
        {instance.weightsBytes !== null && (
          <p className="mt-1 text-[10px] text-slate-500">
            {formatBytes(instance.weightsBytes)} weights ·{" "}
            {formatBytes(instance.memoryBytes)} resident
          </p>
        )}
      </div>

      {instance.error && (
        <p className="mt-3 rounded-lg bg-rose-500/10 px-3 py-2 text-[11px] leading-relaxed text-rose-300">
          {instance.error}
        </p>
      )}

      <footer className="mt-4 flex items-center gap-2 border-t border-ink-700 pt-3">
        <button
          onClick={() => setShowLogs((value) => !value)}
          className="rounded-md px-2.5 py-1.5 text-xs font-medium text-slate-400 transition-colors hover:bg-ink-700 hover:text-slate-200"
        >
          {showLogs ? "Hide logs" : "Logs"}
        </button>
        <a
          href={`http://127.0.0.1:${instance.port}/v1/models`}
          target="_blank"
          rel="noreferrer"
          className={`rounded-md px-2.5 py-1.5 text-xs font-medium transition-colors ${
            isLive
              ? "text-slate-400 hover:bg-ink-700 hover:text-slate-200"
              : "pointer-events-none text-slate-700"
          }`}
        >
          Probe
        </a>
        <button
          onClick={() => onStop(instance.id)}
          disabled={!isLive || busy}
          className="ml-auto rounded-md border border-ink-700 px-2.5 py-1.5 text-xs font-medium text-slate-400 transition-colors hover:border-rose-500/50 hover:text-rose-300 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:border-ink-700 disabled:hover:text-slate-400"
        >
          Stop
        </button>
      </footer>

      {showLogs && (
        <pre className="mt-3 max-h-56 overflow-auto rounded-lg bg-ink-950 p-3 font-mono text-[11px] leading-relaxed text-slate-400">
          {instance.logs.length === 0
            ? "No output yet."
            : instance.logs.join("\n")}
        </pre>
      )}
    </article>
  );
}

/** Compact memory row used inside the card body. */
function MemoryBarInline({
  usedBytes,
  totalBytes,
}: {
  usedBytes: number;
  totalBytes: number;
}) {
  const ratio = totalBytes > 0 ? Math.min(usedBytes / totalBytes, 1) : 0;
  const tone =
    ratio > 0.85
      ? "bg-rose-500"
      : ratio > 0.65
        ? "bg-amber-400"
        : "bg-accent-400";

  return (
    <div className="w-full">
      <div className="mb-1 flex items-baseline justify-between text-[11px] text-slate-400">
        <span>Memory</span>
        <span className="font-mono text-slate-300">
          {formatBytes(usedBytes)}
        </span>
      </div>
      <div className="h-1.5 w-full overflow-hidden rounded-full bg-ink-700">
        <div
          className={`h-full rounded-full transition-all duration-500 ${tone}`}
          style={{ width: `${Math.max(ratio * 100, ratio > 0 ? 2 : 0)}%` }}
        />
      </div>
    </div>
  );
}
