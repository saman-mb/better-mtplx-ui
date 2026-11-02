/** Presentation helpers shared across views. */

const BYTES_PER_GB = 1024 ** 3;

/** Format a byte count as a human-readable size, e.g. `28.4 GB`. */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 GB";
  if (bytes < BYTES_PER_GB) {
    return `${(bytes / 1024 ** 2).toFixed(0)} MB`;
  }
  return `${(bytes / BYTES_PER_GB).toFixed(1)} GB`;
}

/** Format a 0–1 ratio as a percentage string. */
export function formatPercent(ratio: number): string {
  return `${Math.round(ratio * 100)}%`;
}

/** Strip the owner prefix from a Hugging Face repo id for display. */
export function shortRepo(repoId: string): string {
  const parts = repoId.split("/");
  return parts[parts.length - 1] ?? repoId;
}

/** Human label for an instance status. */
export function statusLabel(status: string): string {
  switch (status) {
    case "starting":
      return "Starting";
    case "loading":
      return "Loading";
    case "ready":
      return "Ready";
    case "stopping":
      return "Stopping";
    case "failed":
      return "Failed";
    default:
      return "Stopped";
  }
}
