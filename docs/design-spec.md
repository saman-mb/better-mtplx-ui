# Design Specification: Better MTPLX UI

## 1. Project Overview
A lightweight, parallel-first GUI for managing MTPLX model instances on Apple Silicon. Focus is on orchestration and monitoring rather than chat interaction.

## 2. Tech Stack
- **Runtime Environment:** [Tauri](https://tauri.app/) (Rust backend, native WebView frontend)
- **Frontend Framework:** React.js
- **Styling:** Tailwind CSS
- **Language (Backend):** Rust (for high-performance orchestration and system monitoring)
- **Language (Frontend):** TypeScript
- **Communication:** Tauri Commands (Rust <-> JS)

## 3. Design Language
- **Aesthetic:** Minimalist, "Command Center" style.
- **Theme:** Default Dark Mode.
- **Color Palette:**
    - Background: `#0f172a` (Slate-900)
    - Card Background: `#1e293b` (Slate-800)
    - Accent: `#38bdf8` (Sky-400)
    - Text: `#f8fafc` (Slate-50)
    - Status Colors: Green (Active), Amber (Loading), Red (Error).

## 4. Key UI Components
### 4.1 Sidebar
- **Library:** Browse/Download MTPLX packs.
- **Instances:** List of all active/managed ports.
- **Settings:** Configuration for paths, default ports, and appearance.

### 4.2 Instance Grid (Main View)
A grid of "Instance Cards." Each card represents a running model on a specific port.
- **Header:** Model Name + Close (Stop) button.
- **Body:**
    - Port indicator (e.g., `Port: 8000`).
    - Real-time Memory Gauge (from `sysinfo` via Rust).
    - Status Badge (Active/Idle/Loading).
- **Footer:** Quick actions (Logs, Settings).

## 5. Component Hierarchy
- `App`
    - `Sidebar`
        - `LibraryItem`
        - `InstanceItem`
    - `MainContent`
        - `InstanceGrid`
            - `InstanceCard`
        - `SettingsView`


## 6. Memory attribution

Unified memory is the binding constraint, so the numbers shown must be the
real ones.

| Number | Source | Why |
|---|---|---|
| **Weights** | The engine's own `Memory plan` log line (`weights 77.3G`) | The authoritative unified-memory cost of the model. |
| **Resident** | `sysinfo` process-tree RSS | What is actually paged in right now. Lower than the weights for memory-mapped packs. |
| **Available** | `sysinfo` machine memory | Drives the sidebar gauge. |

External servers (started outside this app) expose no log, so their weight
figure is resolved from the local model cache by matching the server's short
model id against the cached repo ids.

`sysinfo` cannot read the command line of an interpreter whose path contains
spaces — which is exactly the MTPLX runtime venv — so socket ownership is
resolved with `lsof` instead of argv matching.

## 7. External servers

The grid shows two kinds of card:

- **Managed** — started by this app. Solid border, full controls, live logs.
- **External** — already running on the machine. Dashed border, endpoint and
  footprint only, plus a *Stop server* action that goes through
  `mtplx stop --port`.

Ports 8000-8015 are scanned every five ticks. Probes run concurrently and
only a successful `GET /v1/models` counts as a hit.
