import type { InstanceStatus } from "../lib/types";
import { statusLabel } from "../lib/format";

const STYLES: Record<InstanceStatus, string> = {
  starting: "bg-sky-500/15 text-sky-300 ring-sky-500/30",
  loading: "bg-amber-500/15 text-amber-300 ring-amber-500/30",
  ready: "bg-emerald-500/15 text-emerald-300 ring-emerald-500/30",
  stopping: "bg-amber-500/15 text-amber-300 ring-amber-500/30",
  failed: "bg-rose-500/15 text-rose-300 ring-rose-500/30",
  stopped: "bg-slate-500/15 text-slate-400 ring-slate-500/30",
};

const DOTS: Record<InstanceStatus, string> = {
  starting: "bg-sky-400 animate-pulse",
  loading: "bg-amber-400 animate-pulse",
  ready: "bg-emerald-400",
  stopping: "bg-amber-400 animate-pulse",
  failed: "bg-rose-400",
  stopped: "bg-slate-500",
};

/** Small pill showing an instance's lifecycle state. */
export function StatusBadge({ status }: { status: InstanceStatus }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ring-1 ${STYLES[status]}`}
    >
      <span className={`h-1.5 w-1.5 rounded-full ${DOTS[status]}`} />
      {statusLabel(status)}
    </span>
  );
}
