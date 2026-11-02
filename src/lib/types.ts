/** TypeScript mirrors of the Rust types crossing the Tauri boundary. */

export type InstanceStatus =
  "starting" | "loading" | "ready" | "stopping" | "failed" | "stopped";

export interface InstanceView {
  id: string;
  model: string;
  port: number;
  status: InstanceStatus;
  memoryBytes: number;
  tokensPerSecond: number | null;
  error: string | null;
  logs: string[];
  startedAt: number | null;
}

export interface MemorySnapshot {
  totalBytes: number;
  availableBytes: number;
  usedBytes: number;
  usedRatio: number;
}

export interface TickPayload {
  instances: InstanceView[];
  memory: MemorySnapshot;
}

export interface PackValidation {
  contract_present: boolean;
  missing_files: string[];
}

export interface ModelPack {
  repo_id: string;
  path: string;
  size_gb: number | null;
  is_primary: boolean;
  has_runtime_contract: boolean;
  recommended_profile: string | null;
  validation: PackValidation | null;
  delete_command: string | null;
}

export interface ModelsPayload {
  cache_dir: string;
  models: ModelPack[];
}

export interface InspectResult {
  repo_id: string | null;
  compatible: boolean | null;
  has_mtp: boolean | null;
  reasons: string[];
  detail: unknown;
}

export interface DiscoverCard {
  repo: string;
  owner: string;
  branded_name: string;
  downloads: number;
  license: string | null;
}

export interface StatusPayload {
  install: unknown;
  server: unknown;
  diagnostics: unknown;
}

export interface Settings {
  defaultPort: number;
  defaultProfile: string;
  contextWindow: number | null;
  host: string;
  apiKey: string | null;
  schedulerMode: string;
  extraArgs: string[];
  theme: string;
}

export type ViewName = "instances" | "library" | "settings";
