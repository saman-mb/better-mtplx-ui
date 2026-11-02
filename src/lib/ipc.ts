import { invoke } from "@tauri-apps/api/core";
import type {
  DiscoverCard,
  ExternalServer,
  InspectResult,
  InstanceView,
  MemorySnapshot,
  ModelsPayload,
  Settings,
  StatusPayload,
} from "./types";

/** List cached MTPLX packs. */
export const listModels = () => invoke<ModelsPayload>("list_models");

/** Start downloading a pack. */
export const pullModel = (repoId: string) =>
  invoke<void>("pull_model", { repoId });

/** Inspect a pack for runtime-contract and MTP compatibility. */
export const inspectModel = (repoId: string, requireMtp = false) =>
  invoke<InspectResult>("inspect_model", { repoId, requireMtp });

/** Browse MTPLX-branded models published on Hugging Face. */
export const discoverModels = (query?: string, limit = 30) =>
  invoke<DiscoverCard[]>("discover_models", { query: query ?? null, limit });

/** Fetch install / model / integration health. */
export const mtplxStatus = () => invoke<StatusPayload>("mtplx_status");

/** Start a model instance on a port. */
export const startInstance = (
  model: string,
  port: number,
  extraArgs: string[] = [],
) => invoke<InstanceView>("start_instance", { model, port, extraArgs });

/** Stop a model instance. */
export const stopInstance = (id: string) =>
  invoke<void>("stop_instance", { id });

/** Stop an MTPLX server this app did not start, by port. */
export const stopExternalServer = (port: number) =>
  invoke<void>("stop_external_server", { port });

/** Scan for MTPLX servers this app did not start. */
export const discoverServers = () =>
  invoke<ExternalServer[]>("discover_servers");

/** Current view of every instance. */
export const listInstances = () => invoke<InstanceView[]>("list_instances");

/** Machine memory snapshot. */
export const memorySnapshot = () => invoke<MemorySnapshot>("memory_snapshot");

/** Suggest a free port for a new instance. */
export const suggestPort = () => invoke<number>("suggest_port");

/** Read persisted settings. */
export const getSettings = () => invoke<Settings>("get_settings");

/** Persist settings. */
export const setSettings = (settings: Settings) =>
  invoke<Settings>("set_settings", { settings });
