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
