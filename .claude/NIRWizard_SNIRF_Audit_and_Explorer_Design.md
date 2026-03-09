# NIRWizard — SNIRF Parser Audit & Data Explorer Design

## Part 1: SNIRF Parser vs Specification Audit

This section compares the current NIRWizard Rust parser (`snirf_parser.rs` + `snirf.rs` domain model) against the **SNIRF v1.1 specification** ([github.com/fNIRS/snirf](https://github.com/fNIRS/snirf)).

---

### 1.1 Coverage Summary

| SNIRF Field | Spec Status | Parser | Exporter | Notes |
|---|---|---|---|---|
| `/formatVersion` | **required** | ❌ Not read | ❌ Not written | Should read and expose — critical for compatibility checks |
| `/nirs{i}` (indexed root) | **required** | ⚠️ Hardcoded `/nirs` | ⚠️ Hardcoded `/nirs` | Only reads first entry. Hyperscanning files (`/nirs1`, `/nirs2`) will fail silently |
| `metaDataTags` | **required** | ✅ Reads all tags | ✅ Writes all tags | Good — iterates `member_names()` generically |
| `metaDataTags/SubjectID` | **required** | ✅ (via generic) | ✅ | Not validated as present |
| `metaDataTags/MeasurementDate` | **required** | ✅ (via generic) | ✅ | Not validated as ISO 8601 |
| `metaDataTags/MeasurementTime` | **required** | ✅ (via generic) | ✅ | Not validated as ISO 8601 |
| `metaDataTags/LengthUnit` | **required** | ✅ (via generic) | ✅ | Not parsed into domain model — unit is lost |
| `metaDataTags/TimeUnit` | **required** | ✅ (via generic) | ✅ | Same — unit is lost |
| `metaDataTags/FrequencyUnit` | **required** | ✅ (via generic) | ✅ | Same |
| `data{j}` (indexed) | **required** | ⚠️ Hardcoded `data1` | ⚠️ Hardcoded `data1` | Only reads first data block. Multi-block files unsupported |
| `data.dataTimeSeries` | **required** | ✅ | ✅ | Read as `Array2<f64>`, columns mapped to channels |
| `data.time` | **required** | ✅ | ✅ | ⚠️ Does not handle the 2-element `[start, spacing]` compact form |
| `measurementList{k}` | **required** | ⚠️ Partial | ⚠️ Partial | See detailed analysis below — this is the biggest gap |
| `measurementList.sourceIndex` | **required** | ✅ | ✅ | Read as `i32` |
| `measurementList.detectorIndex` | **required** | ✅ | ✅ | Read as `i32` |
| `measurementList.wavelengthIndex` | **required** | ❌ Not read | ✅ Written | **Critical bug**: parser ignores `wavelengthIndex` entirely and assumes first half = HbR, second half = HbO |
| `measurementList.dataType` | **required** | ❌ Not read | ⚠️ Hardcodes `99` | Cannot distinguish raw CW data (type 1) from processed HbO/HbR (type 99999) |
| `measurementList.dataTypeIndex` | **required** | ❌ Not read | ❌ Not written | Needed for TD, DCS, and processed data types |
| `measurementList.wavelengthActual` | optional | ❌ | ❌ | |
| `measurementList.dataUnit` | optional | ❌ | ❌ | |
| `measurementList.dataTypeLabel` | optional | ❌ | ❌ | Important for processed data — "HbO", "HbR", "dOD", etc. |
| `measurementList.sourcePower` | optional | ❌ | ❌ | |
| `measurementList.detectorGain` | optional | ❌ | ❌ | |
| `measurementList.moduleIndex` | optional | ❌ | ❌ | Needed for modular systems (e.g., MOBI) |
| `measurementList.sourceModuleIndex` | optional | ❌ | ❌ | |
| `measurementList.detectorModuleIndex` | optional | ❌ | ❌ | |
| `stim{j}` | optional | ✅ | ✅ | Iterates correctly with loop |
| `stim.name` | req if stim | ✅ | ✅ | |
| `stim.data` | req if stim | ✅ | ✅ | Read as `Array2`, rows filtered for ≥3 cols |
| `stim.dataLabels` | optional | ❌ | ❌ | Loses column annotation for extra stim columns |
| `probe` | **required** | ✅ | ✅ | |
| `probe.wavelengths` | **required** | ✅ | ✅ | |
| `probe.wavelengthsEmission` | optional | ❌ | ❌ | Needed for fluorescence data |
| `probe.sourcePos2D` | req (one of 2D/3D) | ✅ | ✅ | Falls back to 3D slice if absent — good |
| `probe.sourcePos3D` | req (one of 2D/3D) | ✅ | ✅ | |
| `probe.detectorPos2D` | req (one of 2D/3D) | ✅ | ✅ | Falls back to 3D slice — good |
| `probe.detectorPos3D` | req (one of 2D/3D) | ✅ | ✅ | |
| `probe.frequencies` | optional | ❌ | ❌ | FD systems |
| `probe.timeDelays` | optional | ❌ | ❌ | TD gated systems |
| `probe.timeDelayWidths` | optional | ❌ | ❌ | TD gated systems |
| `probe.momentOrders` | optional | ❌ | ❌ | TD moment systems |
| `probe.correlationTimeDelays` | optional | ❌ | ❌ | DCS systems |
| `probe.correlationTimeDelayWidths` | optional | ❌ | ❌ | DCS systems |
| `probe.sourceLabels` | optional | ❌ | ❌ | Parser generates "S1", "S2" etc. instead |
| `probe.detectorLabels` | optional | ❌ | ❌ | Parser generates "D1", "D2" etc. |
| `probe.landmarkPos2D` | optional | ❌ | ❌ | Anatomical landmarks — important for registration |
| `probe.landmarkPos3D` | optional | ❌ | ❌ | Same |
| `probe.landmarkLabels` | optional | ❌ | ❌ | "Nasion", "Inion", "Cz", etc. |
| `probe.coordinateSystem` | optional | ❌ | ❌ | "MNI152NLin2009bAsym", "CapTrak", etc. |
| `probe.coordinateSystemDescription` | optional | ❌ | ❌ | |
| `probe.useLocalIndex` | optional | ❌ | ❌ | Modular systems |
| `aux{j}` | optional | ✅ | ✅ | Loop iteration, reads name/unit/data/time |
| `aux.name` | req if aux | ✅ | ✅ | |
| `aux.dataTimeSeries` | req if aux | ✅ | ✅ | ⚠️ Read as 1D `Vec<f64>`, but spec says 2D `[T × C]` |
| `aux.dataUnit` | optional | ✅ | ✅ | |
| `aux.time` | req if aux | ✅ | ✅ | |
| `aux.timeOffset` | optional | ❌ | ❌ | |

---

### 1.2 Critical Issues

#### Issue 1: The HbO/HbR Half-Split Assumption

This is the most significant problem. The current parser does:

```rust
let half = ts.data.len() / 2;
// ...
hbr: std::mem::take(&mut ts_data[i]),          // first half
hbo: std::mem::take(&mut ts_data[i + half]),   // second half
```

This assumes the `dataTimeSeries` matrix is organized as `[HbR channels | HbO channels]` — first half is one wavelength, second half is the other. This is **not guaranteed by the SNIRF spec**. The correct approach is:

1. Read `measurementList{k}/wavelengthIndex` for every column `k`.
2. Use the `wavelengthIndex` to look up which wavelength from `probe.wavelengths` this column corresponds to.
3. Group columns by `(sourceIndex, detectorIndex)` pairs, matching them by `wavelengthIndex`.

Some real-world SNIRF files interleave wavelengths per source-detector pair (e.g., `[S1-D1@690, S1-D1@830, S1-D2@690, S1-D2@830, ...]`). The current parser will silently produce wrong results for these files.

Furthermore, the parser also assumes exactly 2 wavelengths and that higher wavelength = HbO. This breaks for:
- **Raw intensity data** (dataType=1): columns are raw optical density, not HbO/HbR.
- **3+ wavelength systems** (e.g., 690/780/830 nm setups).
- **Processed data** where `dataTypeLabel` explicitly says "HbO" or "HbR".

#### Issue 2: No `dataType` Awareness

The parser treats all data as if it were processed HbO/HbR concentrations. But a SNIRF file can contain raw CW amplitude (dataType=1), optical density, frequency domain data, or processed hemoglobin — the `dataType` and `dataTypeLabel` fields tell you which. For a data explorer, displaying "HbO" labels on raw intensity data would be misleading.

#### Issue 3: Hardcoded `/nirs` and `data1`

Only the first `/nirs` entry and first `data1` block are read. Multi-dataset files (hyperscanning), multi-run files, and files with multiple data blocks are silently truncated.

#### Issue 4: Missing `formatVersion`

Not reading `/formatVersion` means the parser cannot warn about unsupported future spec versions or validate the file structure.

#### Issue 5: Exporter writes `dataType = 99` instead of `99999`

The exporter writes `dataType = 99` which is not a defined value in the spec. The correct value for processed data is `99999`. This makes exported files non-compliant.

---

### 1.3 Domain Model Gaps

Current domain model assumptions that limit the data explorer:

| Domain Model Issue | Impact |
|---|---|
| `Wavelengths { hbo_wl, hbr_wl }` — only 2 wavelengths, named as HbO/HbR | Can't represent raw data, 3+ wavelength systems, or fluorescence |
| `Channel { hbo: Vec<f64>, hbr: Vec<f64> }` — bakes in HbO/HbR semantics | A generic channel should just have a data vector + metadata about what it contains |
| No `dataType` / `dataTypeLabel` on channels | Can't distinguish raw vs processed data |
| No `wavelengthIndex` per measurement | Lost during parsing — can't reconstruct which wavelength a column belongs to |
| `Optode { pos_3d, pos_2d }` — requires both | Spec only requires one of 2D/3D; parser handles this but domain model implies both always exist |
| No landmark data on `Probe` | Can't do anatomical registration without landmarks |
| No `coordinateSystem` on `Probe` | Can't interpret positions correctly |
| `AuxiliaryData.data` is `Vec<f64>` (1D) | Spec allows `[T × C]` 2D auxiliary data (e.g., 3-axis accelerometer) |

---

## Part 2: Data Explorer Design Outline

### 2.1 Vision

NIRWizard becomes the **"VS Code of SNIRF files"** — the fastest, most intuitive way to open, inspect, explore, and edit fNIRS data files without MATLAB or Python. Not an analysis tool. Not a preprocessing pipeline. A **data-first explorer** with 3D anatomical context.

### 2.2 Core Panels

The explorer UI is organized into switchable/dockable panels:

#### Panel A — HDF5 Tree Walker (the killer feature)

A raw tree view of the HDF5 file structure, like an "Inspector" panel. Every group and dataset in the file is visible.

- **Tree view** shows the hierarchy: `/formatVersion`, `/nirs/metaDataTags/SubjectID`, `/nirs/data1/measurementList3/wavelengthIndex`, etc.
- Each dataset node shows: name, HDF5 type, shape, and a **value preview** (scalars shown inline, arrays show first few elements + shape).
- **Click a dataset** → right panel shows the full value. For arrays, show a scrollable table. For 2D arrays, show both table and a mini chart preview.
- **Editable values**: click the value of a scalar dataset (string, int, float) to edit it inline. Click "Save" to write back to the HDF5 file.
- **Color coding**: green for required fields present, yellow for optional fields present, red/missing for required fields absent (validated against SNIRF spec).
- This directly solves the HbO/HbR identification problem — the user can inspect `measurementList{k}/wavelengthIndex` values and see exactly what each column maps to, and fix it if the file was written incorrectly.

```
📁 /
├── 📄 formatVersion = "1.0"          [string]
├── 📁 nirs/
│   ├── 📁 metaDataTags/
│   │   ├── 📄 SubjectID = "sub-01"   [string]
│   │   ├── 📄 MeasurementDate = "2024-03-15" [string]
│   │   ├── 📄 LengthUnit = "mm"      [string]
│   │   └── ...
│   ├── 📁 data1/
│   │   ├── 📊 dataTimeSeries          [f64] [23239 × 72]
│   │   ├── 📊 time                    [f64] [23239]
│   │   ├── 📁 measurementList1/
│   │   │   ├── 📄 sourceIndex = 1     [i32]
│   │   │   ├── 📄 detectorIndex = 1   [i32]
│   │   │   ├── 📄 wavelengthIndex = 1 [i32]   ← this tells you it's 690nm
│   │   │   ├── 📄 dataType = 1        [i32]   ← CW amplitude
│   │   │   └── 📄 dataTypeIndex = 1   [i32]
│   │   ├── 📁 measurementList2/
│   │   │   └── ...
│   │   └── ...
│   ├── 📁 probe/
│   │   ├── 📊 wavelengths = [690.0, 830.0]  [f64] [2]
│   │   ├── 📊 sourcePos3D             [f64] [8 × 3]
│   │   ├── 📊 detectorPos3D           [f64] [16 × 3]
│   │   └── ...
│   ├── 📁 stim1/
│   │   ├── 📄 name = "Tapping/Left"   [string]
│   │   └── 📊 data                    [f64] [15 × 3]
│   └── 📁 aux1/
│       └── ...
```

#### Panel B — Channel Table

A tabular view of the measurement list, derived from the HDF5 tree:

| # | Source | Detector | Wavelength (nm) | dataType | dataTypeLabel | Column in dataTimeSeries |
|---|--------|----------|----------------|----------|---------------|--------------------------|
| 1 | S1 | D1 | 690 | 1 (CW) | — | col 0 |
| 2 | S1 | D1 | 830 | 1 (CW) | — | col 1 |
| 3 | S1 | D2 | 690 | 1 (CW) | — | col 2 |
| ... | | | | | | |

This table is the **Rosetta Stone** for understanding what's in the file. Sortable, filterable. Click a row → jumps to that channel's time series in Panel C.

#### Panel C — Time Series Viewer

Interactive zoomable chart of selected channels over time. Features:

- Multi-channel overlay (select from Channel Table).
- Event markers drawn as vertical spans (from `stim` data).
- Wavelength-grouped view: show both wavelengths for a source-detector pair side by side.
- If data is processed (dataType=99999 with dataTypeLabel "HbO"/"HbR"): show the canonical red/blue HbO/HbR plot.
- If data is raw: show raw intensity, labeled correctly.
- Zoom, pan, time-range selection.
- Basic signal quality indicators: mean, std, min/max per channel.

#### Panel D — Probe Geometry

Interactive 2D/3D view of the optode layout:

- Sources (red dots), detectors (blue dots), channels (lines between pairs).
- Click a channel line → highlights the corresponding row in Channel Table and shows that time series.
- If `landmarkPos3D` and `landmarkLabels` are present: show anatomical landmarks (Nasion, Inion, Cz, etc.).
- If `coordinateSystem` is present: show coordinate frame label.
- Future: overlay on head atlas mesh (the anatomical registration direction you're already heading).

#### Panel E — File Summary / Metadata

A clean dashboard showing at a glance:

- File name, format version, file size
- Subject ID, measurement date/time
- Number of sources, detectors, channels, timepoints
- Sampling rate (derived from time vector)
- Wavelengths
- Data type (raw CW / processed HbO-HbR / etc.)
- Number of stimulus conditions, with marker counts
- Number of auxiliary channels
- **SNIRF Compliance Check**: list of required fields that are missing or malformed (from the tree walker validation)

---

### 2.3 Architecture: Raw HDF5 Layer vs Domain Layer

The key insight for the data explorer is to separate two layers:

```
┌─────────────────────────────────────────────┐
│  Layer 1: Raw HDF5 Access (new)             │
│  - Generic tree walker / editor             │
│  - No SNIRF semantics assumed               │
│  - Read/write any group, dataset, attribute │
│  - Powers Panel A (HDF5 Tree Walker)        │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│  Layer 2: SNIRF Domain Model (refactored)   │
│  - Interprets HDF5 contents as fNIRS data   │
│  - Uses wavelengthIndex, dataType properly  │
│  - Powers Panels B/C/D/E                    │
│  - Validation against spec                  │
└─────────────────────────────────────────────┘
```

Layer 1 is the new piece — a generic HDF5 browser that works on any `.snirf` (or even `.hdf5`) file. This is what lets users inspect and fix files directly. Layer 2 is the existing parser, refactored to be correct.

---

### 2.4 Refactored Domain Model (Proposal)

```rust
/// A single measurement column in dataTimeSeries
pub struct Measurement {
    pub source_index: usize,       // 1-based per spec
    pub detector_index: usize,     // 1-based per spec
    pub wavelength_index: usize,   // index into probe.wavelengths
    pub data_type: i32,            // 1=CW, 99999=processed, etc.
    pub data_type_label: Option<String>,  // "HbO", "HbR", "dOD", etc.
    pub data_type_index: i32,
    pub data_unit: Option<String>,
    pub data: Vec<f64>,            // the actual column from dataTimeSeries
    // optional fields
    pub wavelength_actual: Option<f64>,
    pub source_power: Option<f64>,
    pub detector_gain: Option<f64>,
    pub module_index: Option<i32>,
}

/// A data block — one /nirs{i}/data{j}
pub struct DataBlock {
    pub time: Vec<f64>,
    pub measurements: Vec<Measurement>,
}

/// The probe geometry
pub struct Probe {
    pub wavelengths: Vec<f64>,
    pub wavelengths_emission: Option<Vec<f64>>,
    pub sources: Vec<Optode>,
    pub detectors: Vec<Optode>,
    pub landmarks: Option<Vec<Landmark>>,
    pub coordinate_system: Option<String>,
    pub coordinate_system_description: Option<String>,
    pub use_local_index: Option<i32>,
    // FD/TD/DCS fields as needed
}

pub struct Landmark {
    pub label: String,
    pub pos_2d: Option<[f64; 2]>,
    pub pos_3d: Option<[f64; 3]>,
}

/// Top-level SNIRF — supports multiple /nirs entries
pub struct SNIRF {
    pub format_version: String,
    pub nirs_entries: Vec<NirsEntry>,  // /nirs1, /nirs2, ...
}

pub struct NirsEntry {
    pub metadata: Metadata,
    pub data_blocks: Vec<DataBlock>,   // data1, data2, ...
    pub probe: Probe,
    pub events: Vec<Event>,
    pub auxiliaries: Vec<AuxiliaryData>,
}
```

Key differences from current model:
- `Measurement` replaces `Channel` — no HbO/HbR assumption baked in
- `wavelengthIndex`, `dataType`, `dataTypeLabel` are preserved per-measurement
- Multiple `/nirs` entries and `data` blocks are supported
- `Probe` includes landmarks and coordinate system
- Grouping measurements into "HbO/HbR channel pairs" becomes a **view-layer concern**, not a parsing concern

---

### 2.5 Interaction: Solving the HbO/HbR Problem via the UI

With the raw HDF5 tree walker + the correct domain model, the user's workflow for the wavelength-identification problem becomes:

1. **Open file** → Panel A shows the HDF5 tree, Panel E shows the summary.
2. **Panel B** (Channel Table) shows each measurement with its `wavelengthIndex` and wavelength in nm. The user can immediately see which columns are at which wavelength.
3. If the file is **raw data** (dataType=1), the UI labels channels as "S1-D1 @ 690nm" and "S1-D1 @ 830nm" — not as HbO/HbR (because it isn't yet).
4. If the file is **processed data** (dataType=99999) with `dataTypeLabel` present, the UI labels channels as "S1-D1 HbO" and "S1-D1 HbR".
5. If the file has **ambiguous or missing metadata**, the HDF5 tree walker lets the user inspect and fix the values directly, then re-parse.

---

### 2.6 Milestone Roadmap

| Milestone | Features | Priority |
|---|---|---|
| **M1: Correct Parser** | Read `wavelengthIndex`, `dataType`, `dataTypeLabel` per measurement. Support `/nirs{i}` and `data{j}` indexing. Read `formatVersion`. Fix exporter `dataType` to `99999`. | 🔴 Critical |
| **M2: HDF5 Tree Walker** | Generic tree view of any `.snirf` file. Value preview for scalars and small arrays. Read-only initially. | 🔴 Critical |
| **M3: Channel Table** | Tabular view of measurements with all spec fields. Sortable, filterable. Click-to-select. | 🟡 High |
| **M4: Time Series Viewer** | Zoomable chart, multi-channel overlay, event markers. Correct labeling based on `dataType`. | 🟡 High |
| **M5: Probe Geometry** | 2D/3D optode plot with interactive selection. | 🟡 High |
| **M6: HDF5 Editor** | Inline editing of scalar values. Write-back to file. | 🟢 Medium |
| **M7: SNIRF Validator** | Check all required fields, flag type mismatches, report compliance. | 🟢 Medium |
| **M8: Landmark Support** | Parse and display `landmarkPos3D`, `landmarkLabels`. | 🟢 Medium |
| **M9: Multi-nirs / Multi-data** | Full support for hyperscanning and multi-block files. | 🟢 Medium |
| **M10: Atlas Overlay** | Map optodes onto standard head atlas (Colin27/MNI152). | 🔵 Future |

---

### 2.7 Competitive Positioning

| Feature | Homer3/AtlasViewer | MNE-NIRS | Satori (NIRx) | **NIRWizard** |
|---|---|---|---|---|
| Platform | MATLAB | Python | Proprietary | **Native (Tauri)** |
| License cost | MATLAB ($940+/yr) or compiled | Free | Paid | **Free & open source** |
| SNIRF support | Full read/write | Full read | Partial | Full read/write (goal) |
| Raw HDF5 inspection | No | No | No | **Yes — tree walker + editor** |
| Installation | MATLAB setup or compiled | pip + dependencies | Installer | **Single binary download** |
| 3D probe view | AtlasViewer (separate app) | Via matplotlib | Yes | **Integrated (WebGL)** |
| SNIRF validation | No | Partial | No | **Yes (planned)** |
| HDF5 field editing | No (requires h5py/HDFView) | No | No | **Yes (planned)** |
| Subject-specific anatomy | AtlasViewer + FreeSurfer | Limited | No | **Planned** |

The gap NIRWizard fills: **no existing tool lets you open a SNIRF file and immediately see both the raw HDF5 structure and the interpreted fNIRS data side by side, in a native app, without MATLAB or Python.**
