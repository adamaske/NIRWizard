# NIRWizard — CLAUDE.md

## Project Overview

NIRWizard is a desktop application for loading and analyzing fNIRS (functional Near-Infrared Spectroscopy) data stored in the **SNIRF** (Shared Near Infrared Spectroscopy Format) file format. Built with Tauri 2 + Svelte 5 + Rust.

## Tech Stack

| Layer     | Technology                        |
|-----------|-----------------------------------|
| Frontend  | Svelte 5.45, Vite 7               |
| Desktop   | Tauri 2.10                        |
| Backend   | Rust (edition 2021, MSRV 1.77.2)  |
| Data I/O  | HDF5 via `hdf5-metno` crate       |
| Numerics  | `ndarray` 0.15                    |
| Serde     | `serde` + `serde_json`            |

## Commands

```bash
# Development (starts Vite + Tauri together)
npx tauri dev

# Production build
npx tauri build

# Frontend only
npm run dev
npm run build
```

## Project Structure

```
NIRWizard/
├── src/                          # Svelte frontend
│   ├── App.svelte                # Root layout & event listeners
│   └── lib/components/
│       ├── controlpanel.svelte   # Load sNIRF button
│       ├── menubar.svelte        # Dropdown menu bar
│       └── statusbar.svelte      # File metadata display
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri entry, commands: load_snirf, get_snirf_summary
│   │   ├── lib.rs                # Plugin setup (logging)
│   │   ├── state.rs              # AppState (RwLock<Session>)
│   │   ├── domain/
│   │   │   ├── snirf.rs          # Core data structs: SNIRF, Probe, TimeSeriesData, etc.
│   │   │   └── biosignals.rs     # (stub)
│   │   └── io/
│   │       └── snirf_parser.rs   # HDF5 parsing logic
│   └── tauri.conf.json           # App config (name, window, bundle)
├── package.json
└── vite.config.js
```

## Architecture

### Frontend → Backend Communication

- **Tauri commands** (invoke): `load_snirf`, `get_snirf_summary`
- **Tauri events** (listen): `snirf-loaded` emitted by backend after parsing

### Data Flow

```
User clicks "Load sNIRF"
  → Tauri file dialog (tauri-plugin-dialog)
  → invoke("load_snirf", { path })
  → Rust: parse HDF5 → update AppState
  → emit("snirf-loaded", SnirfSummary)
  → Svelte: update statusbar props
```

### State Management

- Backend: `AppState` wraps `RwLock<Session>` (thread-safe, managed by Tauri)
- Frontend: reactive Svelte variables; synced via events on mount

## Domain Model (Rust)

```rust
SNIRF {
    fd: FileDescriptor { path, name },
    timeseries: TimeSeriesData { time: Vec<f64>, data: Vec<Vec<f64>> },
    probe: Probe { sources: Vec<Vector3D>, detectors: Vec<Vector3D> },
    biosignals: BiosignalData { time, auxilaries: Vec<AuxiliaryData> },
}

// Sent to frontend
SnirfSummary { filename, channels, sampling_rate, duration }
```

## HDF5 Paths (SNIRF spec)

| HDF5 path                     | Mapped to                     |
|-------------------------------|-------------------------------|
| `nirs/data1/time`             | `TimeSeriesData.time`         |
| `nirs/data1/dataTimeSeries`   | `TimeSeriesData.data`         |
| Probe / aux                   | TODO (placeholder impl)       |

## UI Layout

```
┌──────────────────────────────┐
│ MenuBar                      │  (file, edit, preprocessing, analysis, view, help)
├──────────────────────────────┤
│                              │
│   ControlPanel               │  (main workspace — Load sNIRF button)
│                              │
├──────────────────────────────┤
│ StatusBar (channels, rate, …)│
└──────────────────────────────┘
```

Styling: dark theme (`#0f0f1a` bg, `#d0d0e0` text).

## Environment Variable

`NIRWIZARD_DEFAULT_SNIRF` — if set, the backend auto-loads this file on startup (useful during development).

## What's Implemented vs TODO

**Done:**
- SNIRF HDF5 loading (time-series + file metadata)
- Frontend–backend IPC (commands + events)
- Menu bar UI (dropdowns render, handlers are console stubs)
- Status bar with live metadata

**TODO / Stubs:**
- Menu item backend handlers
- Probe (source/detector positions) parsing
- Auxiliary biosignal parsing
- Data visualization
- Preprocessing pipeline (filter, baseline, motion correction)
- Analysis pipeline
