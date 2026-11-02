import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { TickPayload } from "../lib/types";

const EMPTY: TickPayload = {
  instances: [],
  memory: {
    totalBytes: 0,
    availableBytes: 0,
    usedBytes: 0,
    usedRatio: 0,
  },
  external: [],
};

/**
 * Subscribe to the backend's one-second tick.
 *
 * The Rust side owns all supervision, so the UI never polls on its own — it
 * just renders whatever the last tick carried.
 */
export function useTick(): TickPayload {
  const [tick, setTick] = useState<TickPayload>(EMPTY);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    listen<TickPayload>("instances://tick", (event) => {
      if (cancelled) return;
      // Tolerate a payload from an older backend (e.g. during a hot reload)
      // that does not carry every field yet.
      setTick({
        instances: event.payload.instances ?? [],
        memory: event.payload.memory ?? EMPTY.memory,
        external: event.payload.external ?? [],
      });
    })
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch(() => {
        /* running outside Tauri (e.g. `vite dev` in a browser) — no live data */
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return tick;
}
