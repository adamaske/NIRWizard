# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NIRWizard is a desktop application for loading and analyzing fNIRS (functional Near-Infrared Spectroscopy) data stored in the **SNIRF** (Shared Near Infrared Spectroscopy Format) file format. Built with Tauri 2 + Svelte 5 + Rust.

## Tech Stack

| Layer     | Technology                                                              |
|-----------|-------------------------------------------------------------------------|
| Frontend  | Svelte 5.45, Vite 7                                                     |
| Desktop   | Tauri 2.10                                                              |
| Backend   | Rust (edition 2021, MSRV 1.77.2)                                        |
| Data I/O  | HDF5 via `hdf5-metno`; MRI/mesh via `neuroformats`, `tobj`             |
| Numerics  | `ndarray` 0.15, `nalgebra` 0.34                                         |
| Geometry  | `parry3d` (collision), `bvh` (ray-casting), `kiddo` (k-d tree)         |
| Serde     | `serde` + `serde_json`                                                  |

## Commands

```bash
# Development (starts Vite + Tauri together)
npx tauri dev

# Production build
npx tauri build

# Frontend only
npm run dev
npm run build

# CLI inspector (dumps SNIRF file info to stdout)
cargo run --bin inspect_snirf -- path/to/file.snirf
```

## Project Structure

```
NIRWizard/
├── src/                                # Svelte frontend
│   ├── App.svelte                      # Root layout, drag-to-resize panels, tab state
│   └── lib/
│       ├── Bars/                       # MenuBar, StatusBar
│       ├── NIRS/Info.svelte            # File metadata / signal info cards
│       ├── ChannelSelection/ChannelSelector2D.svelte  # SVG probe map, click-to-select
│       ├── Plotting/                   # TimeSeries, Spectrogram, Frequency, HRF, Connectivity
│       ├── Anatomy/                    # Viewport.svelte (Three.js 3D), SceneInspector.svelte
│       ├── HDF5/HDF5TreeWalker.svelte  # Raw HDF5 tree browser
│       └── components/                # SceneObjectEditor, TransformEditor, OptodeLayoutEditor, VoxelEditor
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                     # Tauri entry; registers all commands
│   │   ├── setup.rs                    # Tauri setup hook (env var auto-load)
│   │   ├── state.rs                    # AppState (split RwLocks: nirs, anatomy, selection, project, analysis)
│   │   ├── domain/                     # Pure Rust data model (no Tauri deps)
│   │   │   ├── snirf.rs                # SNIRF, NirsEntry, DataBlock, Measurement, Probe, Optode
│   │   │   ├── nirs_view.rs            # NirsView / ChannelView — computed view over raw SNIRF data
│   │   │   ├── probe.rs                # OptodeLayout, Optode3D, ChannelConnection, ProbeDisplaySettings
│   │   │   ├── scene.rs                # SceneObject, Transform, ObjectMeta
│   │   │   ├── mesh.rs                 # Mesh, MeshGeometry, Vertex
│   │   │   ├── anatomy.rs              # SubjectAnatomy (skull/csf/grey/white SceneObjects)
│   │   │   ├── voxel.rs                # VoxelVolume
│   │   │   └── error.rs                # SnirfError
│   │   ├── io/                         # File I/O (no domain logic)
│   │   │   ├── snirf_parser.rs         # HDF5 → SNIRF
│   │   │   ├── snirf_exporter.rs       # SNIRF → HDF5
│   │   │   ├── hdf5_parser.rs          # Low-level HDF5 helpers
│   │   │   ├── mesh_importer.rs        # OBJ → Mesh (via tobj)
│   │   │   └── anatomy_importer.rs     # MRI → SubjectAnatomy (via neuroformats)
│   │   └── commands/                   # Tauri command handlers
│   │       ├── mod.rs                  # import_snirf, export_snirf, get_snirf_summary (+ re-exports)
│   │       ├── probe.rs                # get_probe_layout, set_selected_channels
│   │       ├── timeseries.rs           # get_timeseries_data, set_cursor_timepoint
│   │       ├── scene.rs                # anatomy geometry/transform/opacity, optode layout 3D
│   │       ├── anatomy.rs              # load_mri
│   │       ├── voxel.rs                # list/info/slice voxel volumes
│   │       ├── events.rs               # event-related commands (stub)
│   │       └── mesh.rs                 # mesh-related commands (stub)
│   └── tauri.conf.json
└── package.json
```

## Architecture

### Frontend → Backend Communication

| Direction        | Mechanism           |
|------------------|---------------------|
| Frontend → Rust  | `invoke`            |
| Rust → Frontend  | `emit` / `listen`   |

**All registered commands** (see `main.rs`):
```
import_snirf, export_snirf, get_snirf_summary
timeseries::get_timeseries_data, timeseries::set_cursor_timepoint
probe::get_probe_layout, probe::set_selected_channels
scene::get_anatomy_geometry, scene::set_anatomy_transform, scene::set_anatomy_opacity
scene::get_optode_layout_3d, scene::set_optode_layout_transform, scene::set_optode_layout_settings
anatomy::load_mri
voxel::list_voxel_volumes, voxel::get_voxel_volume_info, voxel::get_voxel_slice
```

**Events emitted by Rust**: `snirf-loaded` (payload: `SnirfSummary`)

### Adding a New Tauri Command

1. Write `#[tauri::command] pub fn my_cmd(...)` in the appropriate `commands/*.rs` file.
2. `pub use` it in `commands/mod.rs` if needed.
3. Reference the **full submodule path** in `main.rs`:
   ```rust
   commands::my_module::my_cmd,   // correct — macro finds __cmd__my_cmd here
   // commands::my_cmd,           // wrong — pub use re-exports fn but not __cmd__ symbol
   ```

### State Management

- **Backend**: `AppState` has separate `RwLock`-guarded sub-states: `nirs` (`NirsState` — SNIRF + optode layout), `anatomy` (`AnatomyState` — subject anatomy + voxel volumes), `selection` (`SelectionState` — selected channels), `project` (`ProjectState` — data directory), `analysis` (`AnalysisState` — stub).
- **Frontend**: reactive Svelte `let` variables; components sync via `snirf-loaded` event on `onMount`. Layout sizes persisted to `localStorage` under key `nirwizard_layout`.

## Domain Model (Rust)

### SNIRF data model (`domain/snirf.rs`)
```rust
SNIRF {
    format_version: String,
    file_descriptor: FileDescriptor { filepath, filename },
    nirs_entries:    Vec<NirsEntry>,
}
NirsEntry {
    metadata:    Vec<MetadataTag>,
    data_blocks: Vec<DataBlock>,
    probe:       Probe { wavelengths, sources, detectors, landmarks, coordinate_system, ... },
    events:      Vec<Event { name, markers: Vec<EventMarker { onset, duration, value }> }>,
    auxiliaries: Vec<AuxiliaryData { name, unit, data, time, time_offset }>,
}
DataBlock { time: Vec<f64>, measurements: Vec<Measurement> }
Measurement {
    source_index, detector_index, wavelength_index,
    data_type: i32,         // 1 = CW raw, 9999 = processed (HbO/HbR)
    data_type_label: String, // "HbO", "HbR", "dOD HbO", etc.
    data: Vec<f64>,
    // optional: wavelength_actual, source_power, detector_gain, module_index
}
Optode { id, name, pos_3d: Vector3<f64>, pos_2d: Vector2<f64> }
```

### NirsView — computed view layer (`domain/nirs_view.rs`)
`NirsView<'a>` borrows a `NirsEntry` and provides a channel-oriented API without copying data.
- `ChannelView` groups all measurements for a unique source–detector pair, with indices into `DataBlock.measurements`.
- Use `NirsView::channels_block0()` to get the channel list for the primary data block.
- `NirsView::signal_kind(measurement)` → `SignalKind::Hemoglobin(HemoType)` or `RawAtWavelength(nm)`.
- `NirsView::hbo_data(channel)` / `hbr_data(channel)` — convenience access for processed files.
- `NirsView::data_kind()` → `DataKind::ProcessedHemoglobin | RawCW | Empty`.

**SNIRF uses 1-based source/detector indices** in `measurementList`. These are stored as-is in `Measurement` and `ChannelView`. Use `ChannelView::source_idx_0based()` / `detector_idx_0based()` when indexing into probe optode vecs.

### 3D Probe & Scene

```rust
OptodeLayout { sources: Vec<Optode3D>, detectors: Vec<Optode3D>, channels: Vec<ChannelConnection>,
               settings: ProbeDisplaySettings, transform: Transform }
Optode3D     { id, name, position: Vector3<f64> }    // raw mm-scale SNIRF pos
ChannelConnection { id, source_idx, detector_idx }   // 0-based into sources/detectors
Transform    { position, rotation (Euler°), scale }  // composed as T*R*S

SubjectAnatomy { skull, csf, grey_matter, white_matter: Option<SceneObject> }
SceneObject    { meta: ObjectMeta, mesh: Mesh, transform, visible, opacity }
MeshGeometry   { verts: Vec<Vertex { position, normal }>, indices: Vec<u32> }
```

Mesh data is sent to the frontend as a flat `MeshGeometryPayload { positions: Vec<f32>, normals: Vec<f32>, indices: Vec<u32> }` for direct consumption by Three.js `BufferGeometry`.

Anatomy layers are addressed by string key: `"skull"`, `"csf"`, `"grey_matter"`, `"white_matter"`.

## HDF5 Paths (SNIRF spec)

| HDF5 path                              | Mapped to                                        |
|----------------------------------------|--------------------------------------------------|
| `/nirs/metaDataTags/*`                 | `NirsEntry.metadata`                             |
| `/nirs/probe/wavelengths`              | `Probe.wavelengths`                              |
| `/nirs/probe/sourcePos2D` / `3D`       | `Optode.pos_2d` / `pos_3d` (sources)            |
| `/nirs/probe/detectorPos2D` / `3D`     | `Optode.pos_2d` / `pos_3d` (detectors)          |
| `/nirs/data1/time`                     | `DataBlock.time`                                 |
| `/nirs/data1/dataTimeSeries`           | `Measurement.data` (one column per measurement) |
| `/nirs/data1/measurementList{i}`       | `Measurement` (source/detector/wavelength/type) |
| `/nirs/stim{i}`                        | `Event` (name + markers)                         |
| `/nirs/aux{i}`                         | `AuxiliaryData`                                  |

## UI Layout

```
┌──────────────────────────────────────────────────────────┐
│ MenuBar                                                   │
├──────────────────────────────────────────────────────────┤
│ [Info|HDF5 tabs]  ┊  TimeSeries (ECharts)                │  ← top row
│                   ┊  ─────────────────────────────────── │  ← inner divider
│                   ┊  Spectrogram | Frequency (tabs)      │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤  ← row divider
│ ChannelSelector2D ┊  HRF_CM_Panel  ┊  AnatomyViewport   │  ← bottom row
├──────────────────────────────────────────────────────────┤
│ StatusBar                                                 │
└──────────────────────────────────────────────────────────┘
```

- Dark theme (`#0f0f1a` bg, `#d0d0e0` text, `#1c1c2e` borders)
- All panels are drag-resizable; sizes persisted to `localStorage`
- `AnatomyViewport` uses Three.js for 3D mesh rendering

## Svelte Component Patterns

### Filling a flex parent correctly

```css
/* Use flex: 1, not width/height: 100% */
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

Always add `width="100%"` and `height="100%"` as HTML **attributes** on `<svg>`, not just CSS — SVG has a built-in intrinsic size (300×150 px) that the flex algorithm uses when no attribute is present.

### Svelte reactivity with Set

```js
// Wrong — Svelte can't see mutation of an existing Set
selectedIds.add(id);

// Correct — reassign to trigger reactivity
selectedIds = new Set(selectedIds);

// Reference selectedIds directly in the template (not through a wrapper function)
stroke={selectedIds.has(ch.id) ? COLOR_A : COLOR_B}  // ✓ reactive
stroke={selected(ch.id) ? COLOR_A : COLOR_B}          // ✗ not reactive
```

## Environment Variable

`NIRWIZARD_DEFAULT_SNIRF` — if set, the backend auto-loads this file on startup.

```bash
$env:NIRWIZARD_DEFAULT_SNIRF = "C:\path\to\file.snirf"; npx tauri dev
```

## What's Implemented vs TODO

**Done:**
- Full SNIRF HDF5 parsing and export
- SNIRF domain model with flexible `NirsView`/`ChannelView` abstraction (supports raw CW and processed HbO/HbR)
- Frontend–backend IPC (commands + events)
- Drag-resizable panel layout with `localStorage` persistence
- Info panel (file metadata cards), StatusBar
- ChannelSelector2D: SVG probe map with zoom/pan, click/ctrl-click select
- TimeSeries, Spectrogram, Frequency, HRF/connectivity plots
- 3D Anatomy viewport (Three.js): skull/csf/grey/white matter mesh layers
- SceneInspector: transform/opacity controls per anatomy layer
- MRI loading (`load_mri`), OBJ mesh import
- 3D optode layout with transform and display settings
- Voxel volume loading, info, and slice extraction
- HDF5 tree walker UI

**TODO / Stubs:**
- `commands/events.rs`, `commands/mesh.rs` — stubs
- DataPlotter integration with ChannelSelector (plot selected channels vs. channel 0)
- Menu item backend handlers (preprocessing, analysis)
- Preprocessing pipeline (filter, baseline correction, motion correction)
- Analysis pipeline (GLM, connectivity, etc.)
