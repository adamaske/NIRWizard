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
│   ├── App.svelte                # Root layout, drag-to-resize panels, event wiring
│   └── lib/components/
│       ├── menubar.svelte        # Dropdown menu bar
│       ├── infopanel.svelte      # File metadata / signal info cards
│       ├── ChannelSelector.svelte# SVG probe map — click channels to select
│       ├── DataPlotter.svelte    # ECharts time-series plot (HbO / HbR)
│       └── statusbar.svelte      # File metadata one-liner at the bottom
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri entry point; registers all commands
│   │   ├── lib.rs                # Plugin setup (logging)
│   │   ├── state.rs              # AppState (RwLock<Session>)
│   │   ├── domain/
│   │   │   ├── snirf.rs          # Core structs: SNIRF, Probe, Channel, Optode, etc.
│   │   │   └── biosignals.rs     # (stub)
│   │   ├── io/
│   │   │   └── snirf_parser.rs   # Full HDF5 parser (probe, channels, events, aux)
│   │   └── commands/
│   │       ├── mod.rs            # Re-exports all commands
│   │       ├── summary.rs        # load_snirf, get_snirf_summary → SnirfSummary
│   │       ├── probe.rs          # get_probe_layout, set_selected_channels
│   │       └── timeseries.rs     # get_timeseries_data
│   └── tauri.conf.json           # App config (name, window, bundle)
├── package.json
└── vite.config.js
```

## Architecture

### Frontend → Backend Communication

| Direction        | Mechanism           | Commands / Events                                              |
|------------------|---------------------|----------------------------------------------------------------|
| Frontend → Rust  | `invoke`            | `load_snirf`, `get_snirf_summary`, `get_probe_layout`, `set_selected_channels`, `get_timeseries_data` |
| Rust → Frontend  | `emit` / `listen`   | `snirf-loaded` (payload: `SnirfSummary`)                       |

### Adding a New Tauri Command

1. Write `#[tauri::command] pub fn my_cmd(...)` in the appropriate `commands/*.rs` file.
2. `pub use` it in `commands/mod.rs`.
3. Reference the **full submodule path** in `main.rs`:
   ```rust
   commands::my_module::my_cmd,   // correct — macro finds __cmd__my_cmd here
   // commands::my_cmd,           // wrong — pub use re-exports fn but not __cmd__ symbol
   ```

### Data Flow

```
User clicks "Load sNIRF"
  → Tauri file dialog (tauri-plugin-dialog)
  → invoke("load_snirf", { path })
  → Rust: parse HDF5 → store in AppState
  → emit("snirf-loaded", SnirfSummary)
  → Svelte: App.svelte updates summary → StatusBar, InfoPanel
  → ChannelSelector: listens to snirf-loaded → invoke("get_probe_layout") → draws SVG probe map

User clicks / drags a channel in ChannelSelector
  → invoke("set_selected_channels", { channelIds: [...] })
  → Rust: logs selection (future: feeds DataPlotter)
```

### State Management

- **Backend**: `AppState` wraps `RwLock<Session>` (thread-safe, Tauri-managed)
- **Frontend**: reactive Svelte `let` variables; components sync via `snirf-loaded` event on `onMount`

## Domain Model (Rust)

```rust
SNIRF {
    fd:          FileDescriptor { path, name },
    metadata:    Metadata { tags: Vec<MetadataTag> },
    wavelengths: Wavelengths { hbo_wl: usize, hbr_wl: usize },
    channels:    ChannelData { time: Vec<f64>, channels: Vec<Channel> },
    probe:       Probe { sources: Vec<Optode>, detectors: Vec<Optode> },
    events:      Events { events: Vec<Event> },
    biosignals:  BiosignalData { auxilaries: Vec<AuxiliaryData> },
}

Channel  { id, name, source_id, detector_id, hbo: Vec<f64>, hbr: Vec<f64> }
Optode   { id, name, pos_2d: Vec2, pos_3d: Vec3 }

// Commands return these lightweight structs to the frontend:
SnirfSummary  { filename, channels, sources, detectors, timepoints,
                sampling_rate, duration, hbo_wavelength, hbr_wavelength,
                events, aux_count }
ProbeLayout   { sources: Vec<OptodePosition>, detectors: Vec<OptodePosition>,
                channels: Vec<ChannelTopology> }
OptodePosition { id, name, x, y }          // 2-D position
ChannelTopology { id, name, source_idx, detector_idx }  // 0-based indices
```

**Important**: SNIRF uses 1-based source/detector indices in `measurementList`.
The parser stores them as-is in `Channel.source_id` / `Channel.detector_id`.
`get_probe_layout` converts to 0-based (`source_id - 1`) before sending to the frontend.

## HDF5 Paths (SNIRF spec)

| HDF5 path                              | Mapped to                              |
|----------------------------------------|----------------------------------------|
| `/nirs/metaDataTags/*`                 | `Metadata.tags`                        |
| `/nirs/probe/wavelengths`              | `Wavelengths`                          |
| `/nirs/probe/sourcePos2D` / `3D`       | `Optode.pos_2d` / `pos_3d` (sources)  |
| `/nirs/probe/detectorPos2D` / `3D`     | `Optode.pos_2d` / `pos_3d` (detectors)|
| `/nirs/data1/time`                     | `ChannelData.time`                     |
| `/nirs/data1/dataTimeSeries`           | `Channel.hbr` (cols 0..N) + `hbo` (cols N..2N) |
| `/nirs/data1/measurementList{i}`       | `Channel.source_id`, `detector_id`     |
| `/nirs/stim{i}`                        | `Event` (name + onset/duration/value)  |
| `/nirs/aux{i}`                         | `AuxiliaryData`                        |

## UI Layout

```
┌─────────────────────────────────────────────┐
│ MenuBar                                      │
├─────────────────────────────────────────────┤
│                                             │
│   DataPlotter (ECharts, HbO + HbR)         │  ← drag row divider
│                                             │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ ChannelSelector (SVG) ┊ InfoPanel (cards)   │  ← drag col divider
├─────────────────────────────────────────────┤
│ StatusBar                                    │
└─────────────────────────────────────────────┘
```

- Styling: dark theme (`#0f0f1a` bg, `#d0d0e0` text, `#1c1c2e` borders)
- All panels are drag-resizable via dividers in `App.svelte`
- Panel sizes are pixel-based after first interaction; seeded at 60% on mount

### ChannelSelector Component

- SVG canvas with world-space transform (`translate + scale` group)
- Sources: red circles (`#dd3333`), Detectors: blue circles (`#3355dd`)
- Unselected channels: grey (`#6e6e8a`), Selected: yellow (`#ffdd00`)
- Click channel line → single select; Ctrl+click → multi-select toggle
- Scroll wheel → zoom centered on cursor; drag background → pan
- Calls `set_selected_channels` on every selection change
- ResizeObserver calls `fitView()` on panel resize

## Svelte Component Patterns

### Filling a flex parent correctly

```css
/* Use flex: 1, not width/height: 100% — percentage dimensions are unreliable
   when the parent's size comes from flex layout rather than an explicit value. */
.component-root {
  flex: 1;
  min-width: 0;   /* allow shrinking below content size */
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
```

### SVG in a flex container

Always add `width="100%"` and `height="100%"` as HTML **attributes** on `<svg>`,
not just CSS — SVG has a built-in intrinsic size (300×150 px) that the flex
algorithm uses when no attribute is present.

### Svelte reactivity with Set

```js
// Wrong — Svelte can't see mutation of an existing Set
selectedIds.add(id);

// Correct — reassign to trigger reactivity
selectedIds = new Set(selectedIds);

// Also: reference selectedIds directly in the template, not through a wrapper
// function, so Svelte tracks it as a dependency:
stroke={selectedIds.has(ch.id) ? COLOR_A : COLOR_B}  // ✓ reactive
stroke={selected(ch.id) ? COLOR_A : COLOR_B}          // ✗ not reactive
```

## Environment Variable

`NIRWIZARD_DEFAULT_SNIRF` — if set, the backend auto-loads this file on startup (useful during development).

```bash
$env:NIRWIZARD_DEFAULT_SNIRF = "C:\path\to\file.snirf"; npx tauri dev
```

## What's Implemented vs TODO

**Done:**
- Full SNIRF HDF5 parsing: metadata, wavelengths, probe 2D/3D, channels (HbO/HbR), events, aux
- Frontend–backend IPC (commands + events)
- Drag-resizable panel layout (DataPlotter top, ChannelSelector + InfoPanel bottom)
- InfoPanel: file info cards (signal, wavelengths, events, aux)
- ChannelSelector: SVG probe map with zoom/pan, click-to-select, Ctrl+multi-select
- DataPlotter: ECharts HbO/HbR time-series (channel 0 on load)
- Status bar with live metadata
- Menu bar UI (dropdowns render, handlers are stubs)

**TODO / Stubs:**
- DataPlotter integration with ChannelSelector (plot selected channels)
- Menu item backend handlers (preprocessing, analysis)
- Probe parsing: 2D positions are loaded but some files may lack them (fallback = zeros)
- Auxiliary biosignal visualization
- Preprocessing pipeline (filter, baseline correction, motion correction)
- Analysis pipeline (GLM, connectivity, etc.)
