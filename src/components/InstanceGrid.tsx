import type { ExternalServer, InstanceView } from "../lib/types";
import { ExternalServerCard } from "./ExternalServerCard";
import { InstanceCard } from "./InstanceCard";

interface Props {
  instances: InstanceView[];
  external: ExternalServer[];
  totalMemoryBytes: number;
  onStop: (id: string) => void;
  onStopExternal: (port: number) => void;
  busy: boolean;
  onOpenLibrary: () => void;
}

/**
 * The main view: every model the user is running, side by side.
 *
 * The grid is the whole point of the app — two cards next to each other make
 * parallel hosting obvious at a glance.
 */
export function InstanceGrid({
  instances,
  external,
  totalMemoryBytes,
  onStop,
  onStopExternal,
  busy,
  onOpenLibrary,
}: Props) {
  const live = instances.filter((instance) =>
    ["starting", "loading", "ready", "stopping"].includes(instance.status),
  );

  // A server we started and then found externally is the same process —
  // never show it twice.
  const managedPorts = new Set(instances.map((instance) => instance.port));
  const foreignServers = external.filter(
    (server) => !managedPorts.has(server.port),
  );

  if (instances.length === 0 && foreignServers.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
        <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-ink-800 text-xl">
          ⬡
        </div>
        <h2 className="text-sm font-semibold text-slate-200">
          No models running
        </h2>
        <p className="max-w-sm text-xs leading-relaxed text-slate-500">
          Pick a pack from the library and start it. Every instance gets its own
          port, so you can run several models at once.
        </p>
        <button
          onClick={onOpenLibrary}
          className="mt-2 rounded-lg bg-accent-500 px-4 py-2 text-xs font-semibold text-ink-950 transition-colors hover:bg-accent-400"
        >
          Browse models
        </button>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto px-6 py-5">
      <div className="mb-4 flex items-baseline gap-3">
        <h1 className="text-sm font-semibold text-slate-100">Running</h1>
        <span className="text-xs text-slate-500">
          {live.length} managed
          {foreignServers.length > 0 && ` · ${foreignServers.length} external`}
          {instances.length - live.length > 0 &&
            ` · ${instances.length - live.length} stopped`}
        </span>
      </div>
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 2xl:grid-cols-3">
        {instances.map((instance) => (
          <InstanceCard
            key={instance.id}
            instance={instance}
            totalMemoryBytes={totalMemoryBytes}
            onStop={onStop}
            busy={busy}
          />
        ))}
        {foreignServers.map((server) => (
          <ExternalServerCard
            key={`external-${server.port}`}
            server={server}
            onStop={onStopExternal}
            busy={busy}
          />
        ))}
      </div>
    </div>
  );
}
