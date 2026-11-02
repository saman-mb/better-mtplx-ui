# Better MTPLX UI

A lightweight, parallel-first desktop GUI for [MTPLX](https://mtplx.com) — the
native MTP speculative-decoding engine for Apple Silicon.

Existing tools treat a model as a conversation partner: one chat window, one
loaded model. **Better MTPLX UI treats a model as a worker**: you browse packs,
launch as many as you like, each on its own port, and watch them side by side.

## Why

- **Parallel hosting out of the box.** Every instance gets its own port. Two
  models serve simultaneously, and the GUI shows both alive at once.
- **Memory is the scarce resource.** Unified memory is shared by GPU and CPU.
  The GUI shows a live gauge per instance and a machine-wide bar, so you can
  see what a load will cost before you commit to it. The weight figure comes
  from the engine's own memory plan, not an estimate.
- **It sees servers you did not start.** An MTPLX server left running by the
  MTPLX desktop app or by a terminal shows up in the same grid, flagged as
  *external*, with its own endpoint and footprint.
- **Lightweight.** Tauri + Rust uses the system WebView. No bundled Chromium,
  no bundled inference runtime — the GUI talks to the `mtplx` CLI you already
  have.

## Requirements

- macOS 14+ on Apple Silicon
- The MTPLX CLI installed at `~/.mtplx/bin/mtplx` (or on `PATH`)
- Node 20+ and Rust 1.80+ to build from source

## Develop

```bash
npm install
npm run tauri:dev
```

Build a distributable bundle:

```bash
npm run tauri:build
```

## Verify

```bash
npm run check                     # TypeScript
npm run build                     # frontend bundle
cd src-tauri && cargo test        # orchestrator integration tests
```

The Rust test-suite drives a stub `mtplx` binary, so it verifies real process
spawning, parallel port binding, log capture and shutdown without loading a
model.

Four runnable examples check the same paths against the real engine:

```bash
cd src-tauri
cargo run --example launch          # start one real instance, verify, stop
cargo run --example parallel         # start two, serve both concurrently
cargo run --example discover         # list MTPLX servers on the loopback
cargo run --example memprobe -- 8000 # explain a memory attribution
```

## Develop the UI without the Rust side

`npm run dev`, then open `/mock.html`. That entry mocks the Tauri IPC bridge
with representative data, so the interface can be iterated on (and
screenshotted) in a plain browser. It is a dev-only entry — `vite build`
builds `index.html`, so none of it reaches the shipped bundle.

## Architecture

| Layer | Where | Responsibility |
|---|---|---|
| Engine bridge | `src-tauri/src/mtplx/` | Resolve and invoke the `mtplx` CLI; parse its JSON payloads |
| Orchestrator | `src-tauri/src/instances/` | Spawn, supervise and stop `mtplx serve` processes |
| System probe | `src-tauri/src/system/` | Unified memory, process-tree and socket-owner attribution |
| Settings | `src-tauri/src/settings/` | Persisted preferences in the app config directory |
| UI | `src/` | React views: Instances, Library, Settings |

The Rust side owns every side effect. The frontend never polls on its own —
it renders a single `instances://tick` event the backend emits once a second.

## Views

- **Instances** — the instance grid. One card per running model, showing its
  port, status, weight footprint, resident memory and throughput, with
  per-instance logs. Servers started elsewhere appear as *external*.
- **Library** — installed packs (ready to run) and the Hugging Face catalog of
  MTPLX-branded models (one click to download).
- **Settings** — default port, serving profile, scheduler mode, context
  window, bind host, API key and raw extra flags.

Activity, System and Forge tabs are out of scope for this release.

## License

MIT
