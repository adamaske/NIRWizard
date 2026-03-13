# NIRwizard — Architecture Review & Critique

**Codebase snapshot:** March 2026  
**Scale:** ~3,200 lines Rust · ~3,000 lines Svelte/JS · ~6,200 total  

---

## Executive Summary

NIRwizard is in a healthy early state. The tech stack choice (Tauri 2 + Svelte 5 + Rust) is strong for a desktop scientific tool — you get native performance for heavy numerics in Rust, a lightweight reactive UI, and cross-platform distribution. The SNIRF domain model is well-designed, the HDF5 parser is thorough, and the `NirsView` borrow-based abstraction is a smart way to avoid data duplication.

However, the architecture has several structural issues that will compound quickly as you add frequency/spectrogram plotting, preprocessing pipelines, MRI registration, MCX simulation, and DOT. The problems aren't in the code quality — they're in **how responsibilities are distributed** and **what's missing between the layers**.

The three highest-priority concerns:

1. **The `Session` struct is a monolithic god-object** that will become unmanageable as features grow.
2. **There is no frontend state management layer** — state lives in ad-hoc `let` variables and event listeners scattered across components.
3. **The command layer conflates data retrieval with data transformation**, making it hard to add derived views (FFT, spectrograms, filtered signals) without creating an explosion of new commands.

---

## 1. Backend Architecture (Rust)

### 1.1 State: The `Session` Problem

```rust
pub struct Session {
    pub data_directory: Option<PathBuf>,
    pub subject_anatomy: Option<SubjectAnatomy>,
    pub voxel_volumes: HashMap<String, VoxelVolume>,
    pub snirf: Option<SNIRF>,
    pub optode_layout: Option<OptodeLayout>,
    pub selected_channels: Vec<usize>,
}
```

This single struct holds **everything**: raw data, derived layouts, UI selection state, file paths, anatomy meshes, and voxel volumes. It's wrapped in one `RwLock`, meaning any write to `selected_channels` blocks reads to `voxel_volumes`. Right now at 6 fields this is fine. Once you add preprocessing state, analysis results, MCX simulation configs, DOT reconstruction outputs, and multiple open files, this becomes the bottleneck for every feature.

**Recommendation — split into domain-specific state containers:**

```rust
pub struct AppState {
    pub project: RwLock<ProjectState>,     // file paths, workspace config
    pub nirs: RwLock<NirsState>,           // SNIRF data, derived views, preprocessing
    pub anatomy: RwLock<AnatomyState>,     // meshes, voxels, MRI registration
    pub selection: RwLock<SelectionState>, // UI-driven: selected channels, time range, cursor
    pub analysis: RwLock<AnalysisState>,   // GLM results, connectivity, DOT outputs
}
```

Each sub-state gets its own lock, so a long-running MCX simulation writing to `analysis` doesn't block the UI from reading `nirs` for plotting. Tauri's `.manage()` can hold multiple state objects, or you keep one wrapper with fine-grained locks inside.

### 1.2 Domain Model: `SNIRF` Ownership vs. Views

The `SNIRF` struct owns all data (including `Vec<f64>` per measurement), and `NirsView` borrows it with `'a` lifetimes. This is a correct and efficient design. However, two issues will surface:

**Problem A — `NirsView` is created on every command call.** In `get_timeseries_data`, a new `NirsView` is constructed each time. This is cheap today, but once you add preprocessing (bandpass filter, motion correction), you'll need to store *derived* timeseries that don't exist in the original SNIRF. The current architecture has no place for this — you'd either mutate the SNIRF (destroying the raw data) or create a parallel data structure that `NirsView` can't borrow from.

**Recommendation:** Introduce a `ProcessedData` layer that sits alongside `SNIRF` in the nirs state:

```rust
pub struct NirsState {
    pub raw: Option<SNIRF>,
    pub pipeline: PreprocessingPipeline, // ordered list of steps applied
    pub processed: Option<ProcessedBlock>, // cached output of pipeline
    pub optode_layout: Option<OptodeLayout>,
}
```

When the user applies a filter, you run the pipeline on `raw`, store results in `processed`, and `NirsView`-like accessors read from `processed` (falling back to `raw` if no processing is applied). This preserves the original data and gives you a clear cache invalidation point.

**Problem B — Hard-coded `nirs_entries[0]` / `block0()` everywhere.** Commands like `get_timeseries_data` always access `.nirs_entries.first()` and `.data_blocks.first()`. SNIRF files can contain multiple nirs entries and data blocks (e.g., multi-run or multi-condition recordings). You should parameterize entry/block selection early, even if the UI only exposes the first one today, to avoid a painful refactor later.

### 1.3 Command Layer: Too Thin, Too Coupled

The current commands are thin wrappers that read state, serialize, and return. This works for "get me the data" but breaks down for derived views. Consider what happens when you implement the frequency plot:

- **Option A (bad):** Add a `get_frequency_data` command that reads SNIRF, computes FFT in Rust, returns the result. Now you have N commands for N plot types, each re-reading state and doing transforms inline.
- **Option B (better):** Separate data access from computation. Commands fetch data; a `compute` module handles transforms; results are cached in state.

**Recommendation — introduce a service/use-case layer between commands and domain:**

```
commands/           → Tauri IPC boundary (thin, just arg parsing + state access)
  ↓
services/           → Business logic orchestration
  ├── nirs.rs       → load, preprocess, export
  ├── analysis.rs   → FFT, spectrogram, GLM, connectivity
  ├── anatomy.rs    → MRI import, registration, segmentation
  └── simulation.rs → MCX config, run, result parsing
  ↓
domain/             → Pure data structures, no I/O
io/                 → File parsing, export (no business logic)
```

This prevents commands from growing into 200-line functions that mix state management, computation, and serialization.

### 1.4 Error Handling

`SnirfError` is well-designed with `thiserror` + `anyhow` layering. But many commands use `.map_err(|e| e.to_string())` to convert errors, which loses the error chain. As the app grows, you'll want structured errors on the frontend to show appropriate UI (e.g., "file format not recognized" vs. "missing probe data" vs. "HDF5 read failed").

**Recommendation:** Define a top-level `AppError` enum that all commands return, with `Serialize` so Tauri can pass structured errors to the frontend:

```rust
#[derive(Debug, Serialize, thiserror::Error)]
pub enum AppError {
    #[error("No data loaded")]
    NoData,
    #[error("SNIRF parse error: {0}")]
    SnirfParse(String),
    #[error("Anatomy import error: {0}")]
    AnatomyImport(String),
    // ...
}
impl From<AppError> for tauri::ipc::InvokeError { ... }
```

### 1.5 Concurrency & Long-Running Tasks

Everything currently runs synchronously on the invoke thread. This is fine for fast reads, but will freeze the UI when you add:

- Preprocessing pipelines (bandpass + motion correction on 50-channel, 30-minute recordings)
- MCX photon simulation (minutes to hours)
- DOT reconstruction (iterative solver)

**Recommendation:** Plan now for an async task system. Tauri 2 supports async commands (`async fn`). For truly long tasks, use a background thread with progress reporting via `app.emit()`:

```rust
#[tauri::command]
async fn run_preprocessing(state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<(), AppError> {
    let raw_data = { state.nirs.read().unwrap().raw.clone() }; // clone data out of lock
    
    tokio::task::spawn_blocking(move || {
        // heavy computation here, emit progress
        app.emit("preprocessing-progress", ProgressPayload { step: 1, total: 5 }).ok();
        // ...
    }).await.map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(())
}
```

### 1.6 Dependency Concerns

- **Two versions of ndarray** (`ndarray` 0.15 and `ndarray16` = 0.16). This is a code smell — pick one and migrate. Having two array crate versions means you'll constantly convert between them.
- **nalgebra** is used for Vec2/Vec3/Matrix4, which is appropriate for 3D scene transforms, but it's heavyweight if you only need small vectors in the SNIRF domain types. Consider whether `[f64; 3]` would suffice for probe positions, reserving nalgebra for the anatomy/scene module.
- **`hdf5-metno`** — confirm this fork is actively maintained. The mainline `hdf5` crate has had activity; you don't want to be stuck on an unmaintained fork when you need newer HDF5 features.

---

## 2. Frontend Architecture (Svelte)

### 2.1 State Management: The Missing Layer

The frontend has no centralized state management. `App.svelte` holds `summary` as a `let` variable. `TimeSeries.svelte` holds `allData` and `selectedIds` locally. `sceneState.js` uses Svelte writable stores but only for the 3D scene. Each component independently calls `invoke()` and `listen()`.

This works at 5 components. At 15-20, you'll have:
- Multiple components listening to `snirf-loaded` and independently calling `invoke("get_timeseries_data")`, duplicating fetches
- No single source of truth for "which channels are selected" (the Rust state, the 2D selector, the TimeSeries component, and the 3D viewport all track this separately)
- No way to coordinate cross-component state changes (e.g., "user applied a filter → all plots need to re-render")

**Recommendation — create a centralized store layer:**

```
src/lib/stores/
  nirsStore.js      → SNIRF summary, timeseries cache, selected channels, data kind
  analysisStore.js  → FFT results, spectrogram cache, preprocessing state
  anatomyStore.js   → loaded layers, transforms, visibility
  selectionStore.js → selected channels, time range, cursor position
```

Components read from stores and dispatch actions. Stores handle invoke calls and caching. One component loads data; all others react to store changes.

```javascript
// nirsStore.js
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const snirfSummary = writable(null);
export const timeseriesCache = writable(null);
export const selectedChannelIds = writable([]);

export const selectedChannelData = derived(
  [timeseriesCache, selectedChannelIds],
  ([$cache, $ids]) => {
    if (!$cache) return [];
    return $ids.length > 0
      ? $cache.channels.filter(ch => $ids.includes(ch.id))
      : [];
  }
);

export async function loadSnirf(path) {
  const summary = await invoke('load_snirf', { path });
  snirfSummary.set(summary);
  const ts = await invoke('get_timeseries_data');
  timeseriesCache.set(ts);
  selectedChannelIds.set(ts.channels.map(ch => ch.id));
}
```

### 2.2 App.svelte: Layout vs. Logic

`App.svelte` is 443 lines handling both panel layout (drag-resize logic, size persistence) and application state (summary, event listeners). These should be separated:

- **`Layout.svelte`** — pure layout shell with resizable panels, no domain knowledge
- **`App.svelte`** — imports Layout, wires up stores, handles global events

The drag-resize logic (lines 59–136) is generic and could be extracted into a reusable `ResizablePanel` component or a Svelte action.

### 2.3 TimeSeries.svelte: Too Much Responsibility

At 421 lines, `TimeSeries.svelte` handles:
- Data fetching and caching (`fetchAndCacheData`)
- Downsampling (LTTB algorithm)
- ECharts configuration for stacked mode
- ECharts configuration for unstacked mode
- Event marker visualization
- Cursor/time input
- Resize handling

This should be decomposed:
- **`timeseriesUtils.js`** — LTTB downsampling, mark line/area builders
- **`useTimeSeries.js`** — data fetching hook (or move to store)
- **`TimeSeriesChart.svelte`** — just the ECharts rendering
- **`TimeSeries.svelte`** — toolbar + chart composition

### 2.4 IPC Pattern: Implicit Contract

The frontend-backend contract is entirely implicit. The frontend calls `invoke("get_timeseries_data")` and hopes the response shape matches what `TimeSeries.svelte` expects. There are no TypeScript types, no schema validation, no shared type definitions.

**Recommendation:** Even if you stay with plain JS (not TypeScript), create a `src/lib/types.js` or `src/lib/ipc.js` that documents and validates the shapes:

```javascript
// src/lib/ipc.js
import { invoke } from '@tauri-apps/api/core';

/** @typedef {{ id: number, name: string, series_a: number[], series_b: number[], series_a_label: string, series_b_label: string }} ChannelPayload */
/** @typedef {{ time: number[], data_kind: string, channels: ChannelPayload[], events: EventPayload[] }} TimeseriesPayload */

export async function getTimeseriesData() {
  /** @type {TimeseriesPayload | null} */
  return invoke('get_timeseries_data');
}
```

Better yet: adopt TypeScript. The Tauri ecosystem has good TS support, and you'll need it as the IPC surface grows.

---

## 3. Cross-Cutting Concerns

### 3.1 Data Transfer: Serialization Bottleneck

`get_timeseries_data` serializes **all channels** as `Vec<f64>` into JSON and sends them over IPC. For a typical 50-channel, 10 Hz, 30-minute recording, that's 50 × 2 (HbO/HbR) × 18,000 timepoints × 8 bytes ≈ **14 MB** of JSON per load. This will be slow and memory-intensive.

**Recommendations:**
- **Lazy loading:** Only send data for selected channels, not all. The current code sends everything and filters on the frontend. Flip this — accept channel IDs as a parameter to `get_timeseries_data`.
- **Pagination/streaming:** For long recordings, send data in time windows rather than the full recording.
- **Binary transfer:** For large payloads, consider Tauri's streaming or binary channel APIs rather than JSON serialization. This matters most for mesh geometry (already done well with flat `Vec<f32>`) and will matter for spectrograms (2D arrays).

### 3.2 Computation Placement: Rust vs. JS

Currently there's an implicit assumption that Rust does I/O and the frontend does visualization, with no computation happening on either side beyond data formatting. As you add features, you need a clear policy:

| Computation | Where | Why |
|---|---|---|
| FFT / PSD | **Rust** | numerically intensive, use `rustfft` or `realfft` |
| STFT / Spectrogram | **Rust** | same, returns 2D array |
| Bandpass filter | **Rust** | DSP, use `biquad` or custom IIR |
| Motion correction (PCA, TDDR) | **Rust** | linear algebra on full dataset |
| LTTB downsampling | **JS** (current) or **Rust** | lightweight, fine either side |
| Chart config building | **JS** | ECharts-specific, must be frontend |
| MCX simulation | **Rust** (spawning external process) | CPU-intensive, needs IPC for progress |
| DOT reconstruction | **Rust** | iterative solver, heavy numerics |

The key principle: **raw data and transforms live in Rust; the frontend only receives pre-computed, display-ready payloads.** Don't send raw timeseries to JS and compute FFT there — send the PSD result.

### 3.3 Testing: Zero Tests

There are no tests in the codebase. For a scientific tool, this is a significant risk. Priority areas for testing:

1. **SNIRF parser** — test against the SNIRF sample files in your `examples/` directory and edge cases (missing optional fields, multiple nirs entries, different data types)
2. **`NirsView` / `build_channel_views`** — verify channel grouping logic, 1-based to 0-based conversion
3. **Signal processing** (once added) — verify FFT output against known test signals, filter frequency response

Rust's built-in test framework makes this straightforward:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_example_snirf() {
        let snirf = parse_snirf("../../examples/example_snirf_data/example.snirf").unwrap();
        assert_eq!(snirf.nirs_entries.len(), 1);
        // ...
    }
}
```

### 3.4 Configuration & Preferences

There's no config system. Settings like default file paths, plot colors, preprocessing defaults, and layout preferences should be persisted. Tauri has `tauri-plugin-store` for this, or you can use a simple JSON config file.

---

## 4. Preparing for Your Roadmap

### 4.1 Immediate: Frequency & Spectrogram Plotting

For the features you're prioritizing right now:

**FFT/PSD command:**
```rust
// New: commands/analysis.rs
#[tauri::command]
pub fn compute_psd(
    channel_ids: Vec<usize>,
    window_size: usize,    // FFT window length
    state: tauri::State<AppState>,
) -> Result<PsdPayload, AppError> { ... }
```

**Spectrogram command:**
```rust
#[tauri::command]
pub fn compute_spectrogram(
    channel_id: usize,
    window_size: usize,
    hop_size: usize,
    state: tauri::State<AppState>,
) -> Result<SpectrogramPayload, AppError> { ... }
```

The spectrogram payload is a 2D float array (time bins × frequency bins). For large recordings this can be several MB — consider sending it as a flat `Vec<f32>` with dimensions rather than nested arrays.

**Crate recommendations:** `rustfft` (pure Rust, no system deps) or `realfft` (wraps rustfft for real-valued input, halving the output). For windowing: `apodize` crate or hand-roll a Hann window.

### 4.2 Medium-Term: Preprocessing Pipeline

Design a pipeline as an ordered list of steps, each with parameters:

```rust
pub enum PreprocessingStep {
    BandpassFilter { low_hz: f64, high_hz: f64, order: usize },
    MotionCorrection { method: MotionMethod, threshold: f64 },
    BaselineCorrection { method: BaselineMethod },
    OpticalDensityConversion,
    BeerLambertTransform { dpf: [f64; 2] },
}

pub struct PreprocessingPipeline {
    pub steps: Vec<PreprocessingStep>,
}
```

Each step is a pure function `fn apply(input: &DataBlock) -> DataBlock`. The pipeline caches intermediate and final results in `NirsState.processed`. When the user modifies a step, invalidate from that step onwards.

### 4.3 Long-Term: MRI Registration, MCX, DOT

These features will each be significant modules. The architecture advice:

- **MRI registration** will need its own state (loaded MRI, affine/nonlinear transforms, registered probe coordinates). Keep it in `AnatomyState`, separate from `NirsState`.
- **MCX simulation** is an external process (MCX is typically a separate binary). Design a `SimulationRunner` that spawns the process, monitors output, and emits progress events. Don't try to embed MCX in-process.
- **DOT reconstruction** connects the simulation forward model to the NIRS data via an inverse solver. This needs both `NirsState` and `AnatomyState` — model this as a service that reads from both but writes to `AnalysisState`.

---

## 5. Prioritized Action Items

### Do Now (before adding frequency/spectrogram)

1. **Split `Session` into domain-specific state containers** with separate locks
2. **Create a Svelte store layer** (`nirsStore.js` at minimum) so components share state
3. **Parameterize `get_timeseries_data`** to accept channel IDs rather than returning everything
4. **Add `rustfft`** dependency and create `services/analysis.rs` for FFT/spectrogram computation
5. **Resolve the dual ndarray versions** — pick 0.16 and migrate

### Do Soon (before preprocessing pipeline)

6. **Introduce the services layer** between commands and domain
7. **Define `AppError`** enum for structured error handling across IPC
8. **Add basic parser tests** against your example SNIRF files
9. **Extract layout logic** from `App.svelte` into a generic resizable panel component
10. **Create `ipc.js`** with typed wrappers for all invoke calls

### Do Before MRI/MCX/DOT

11. **Implement async commands** and background task infrastructure with progress events
12. **Design the `PreprocessingPipeline`** abstraction with caching and invalidation
13. **Add binary transfer** for large payloads (spectrogram matrices, mesh geometry)
14. **Adopt TypeScript** — the IPC surface will be too large for untyped JS
15. **Add a config/preferences system** using `tauri-plugin-store`

---

## 6. What's Working Well

To be clear about what doesn't need changing:

- **SNIRF domain model** — `NirsEntry` / `DataBlock` / `Measurement` is a faithful, well-structured representation of the spec. The distinction between raw CW and processed hemoglobin data via `DataKind` is clean.
- **`NirsView` with borrowing** — zero-copy access to measurement data is the right call for a data-heavy app.
- **HDF5 parser** — thorough, handles edge cases (string encodings, optional fields, fallback positions), good use of `anyhow` context chains.
- **Probe/Scene/Mesh architecture** — the separation between pure geometry (`MeshGeometry`) and spatial indexing (`MeshTopology`) with kdtree and trimesh is well thought out for the registration use case.
- **ECharts integration** — LTTB downsampling, stacked/unstacked modes, event markers, scroll-zoom — this is a solid timeseries viewer.
- **Panel layout system** — drag-resize with localStorage persistence is a nice UX detail.

The codebase is clean, well-commented, and clearly written by someone who understands both the fNIRS domain and the tech stack. The issues above are about **scaling** the architecture, not about fixing broken patterns.
