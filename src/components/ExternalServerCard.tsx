import type { ExternalServer } from "../lib/types";
import { formatBytes } from "../lib/format";

interface Props {
  server: ExternalServer;
  onStop: (port: number) => void;
  busy: boolean;
}

/**
 * A running MTPLX server that this app did not start.
 *
 * These come from the MTPLX desktop app or from a `mtplx serve` run in a
 * terminal. They are real, usable endpoints — the GUI just does not own the
 * process, so it says so instead of pretending it started them.
 */
export function ExternalServerCard({ server, onStop, busy }: Props) {
  return (
    <article className="flex flex-col rounded-xl border border-dashed border-ink-600 bg-ink-800/60 p-4">
      <header className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h3
            className="truncate text-sm font-semibold text-slate-200"
            title={server.modelId}
          >
            {server.modelId}
          </h3>
          <p className="mt-0.5 text-[11px] text-slate-500">
            Started outside this app
          </p>
        </div>
        <span className="inline-flex items-center gap-1.5 rounded-full bg-slate-500/15 px-2.5 py-1 text-xs font-medium text-slate-400 ring-1 ring-slate-500/30">
          <span className="h-1.5 w-1.5 rounded-full bg-slate-400" />
          External
        </span>
      </header>

      <div className="mt-4 grid grid-cols-2 gap-3">
        <div className="rounded-lg bg-ink-900/60 px-3 py-2">
          <div className="text-[10px] uppercase tracking-wider text-slate-500">
            Endpoint
          </div>
          <div className="mt-0.5 font-mono text-xs text-accent-400">
            127.0.0.1:{server.port}
          </div>
        </div>
        <div className="rounded-lg bg-ink-900/60 px-3 py-2">
          <div className="text-[10px] uppercase tracking-wider text-slate-500">
            Memory
          </div>
          <div className="mt-0.5 font-mono text-xs text-slate-200">
            {server.weightsBytes !== null
              ? formatBytes(server.weightsBytes)
              : server.memoryBytes
                ? formatBytes(server.memoryBytes)
                : "—"}
          </div>
        </div>
      </div>

      <footer className="mt-4 flex items-center gap-2 border-t border-ink-700 pt-3">
        <a
          href={`http://127.0.0.1:${server.port}/v1/models`}
          target="_blank"
          rel="noreferrer"
          className="rounded-md px-2.5 py-1.5 text-xs font-medium text-slate-400 transition-colors hover:bg-ink-700 hover:text-slate-200"
        >
          Probe
        </a>
        <button
          onClick={() => onStop(server.port)}
          disabled={busy}
          className="ml-auto rounded-md border border-ink-700 px-2.5 py-1.5 text-xs font-medium text-slate-400 transition-colors hover:border-rose-500/50 hover:text-rose-300 disabled:cursor-not-allowed disabled:opacity-40"
        >
          Stop server
        </button>
      </footer>
    </article>
  );
}
