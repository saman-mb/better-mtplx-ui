import { formatBytes, formatPercent } from "../lib/format";

interface Props {
  usedBytes: number;
  totalBytes: number;
  /** Optional label shown to the left of the bar. */
  label?: string;
  /** Render the percentage next to the label instead of the absolute size. */
  compact?: boolean;
}

/**
 * A single horizontal memory bar.
 *
 * Unified memory is the scarce resource on Apple Silicon, so every place that
 * shows a footprint shows the same visual language.
 */
export function MemoryBar({ usedBytes, totalBytes, label, compact }: Props) {
  const ratio = totalBytes > 0 ? Math.min(usedBytes / totalBytes, 1) : 0;
  const tone =
    ratio > 0.85
      ? "bg-rose-500"
      : ratio > 0.65
        ? "bg-amber-400"
        : "bg-accent-400";

  return (
    <div className="w-full">
      {label && (
        <div className="mb-1 flex items-baseline justify-between text-[11px] text-slate-400">
          <span>{label}</span>
          <span className="font-mono text-slate-300">
            {compact
              ? formatPercent(ratio)
              : `${formatBytes(usedBytes)} / ${formatBytes(totalBytes)}`}
          </span>
        </div>
      )}
      <div className="h-1.5 w-full overflow-hidden rounded-full bg-ink-700">
        <div
          className={`h-full rounded-full transition-all duration-500 ${tone}`}
          style={{ width: `${Math.max(ratio * 100, ratio > 0 ? 2 : 0)}%` }}
        />
      </div>
    </div>
  );
}
