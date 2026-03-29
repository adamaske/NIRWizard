# NIRWizard ideal architecture

Tauri 2 + Svelte 5 + Rust. Focused on fNIRS data inspection: parsing, info,
timeseries, frequency spectrum, spectrograms, channel selection, block navigation.

---

## Dependency rule

Every layer may only import from layers below it. Never up, never lateral.

```
commands/        → services/, state/
services/        → domain/, dsp/, io/
state/           → domain/
io/              → domain/
domain/          → (nothing in crate)
dsp/             → (nothing in crate)
```

---

## File tree

```
src-tauri/src/
├── lib.rs                          # pub mod all modules, run()
├── main.rs                         # calls app_lib::run()
│
├── domain/                         # Pure data types — zero external deps
│   ├── mod.rs
│   ├── snirf.rs                    # SNIRF, NirsEntry, DataBlock, Measurement, Probe, Optode
│   ├── channel.rs                  # ChannelView, build_channel_views(), DataKind, SignalKind
│   ├── probe.rs                    # ProbeLayout, ChannelTopology (no tauri imports)
│   ├── summary.rs                  # SnirfSummary, BlockSummary (moved from commands/)
│   └── error.rs                    # AppError (thiserror + Serialize)
│
├── dsp/                            # Pure signal processing — takes &[f64], returns Vec<f64>
│   ├── mod.rs
│   ├── window.rs                   # WindowType, apply_window()
│   ├── spectrum.rs                 # fft_magnitude(), welch_psd()
│   └── spectrogram.rs              # stft(), cwt_morlet()
│
├── io/                             # File I/O — imports domain/ only
│   ├── mod.rs
│   ├── snirf_parser.rs             # parse_snirf() → domain::Snirf
│   └── snirf_exporter.rs           # export_snirf()
│
├── state/                          # Tauri-managed resources — one .manage() per struct
│   ├── mod.rs
│   ├── session.rs                  # SessionState (loaded SNIRF + cached channel views)
│   └── selection.rs                # SelectionState (selected channels, active block, cursor)
│
├── services/                       # Business logic — pure functions, no Tauri
│   ├── mod.rs
│   ├── session_service.rs          # load/unload SNIRF, build cached views
│   ├── timeseries_service.rs       # assemble channel payloads from domain types
│   └── spectral_service.rs         # bridge domain channels → dsp functions
│
└── commands/                       # Thin Tauri glue — 5-15 lines each
    ├── mod.rs
    ├── file_commands.rs            # import_snirf, export_snirf
    ├── info_commands.rs            # get_summary, get_block_list
    ├── probe_commands.rs           # get_probe_layout
    ├── selection_commands.rs       # set_selected_channels, set_active_block
    ├── timeseries_commands.rs      # get_timeseries_data
    └── spectral_commands.rs        # get_spectrum, get_spectrogram
```

---

## Layer 1: domain/

No `use crate::` imports. No Tauri. No `hdf5`. No `rustfft`.
Only `serde`, `thiserror`, and math types (`nalgebra`).

### domain/snirf.rs

```rust
use nalgebra::{Vector2, Vector3};

/// Root of a parsed SNIRF file.
pub struct Snirf {
    pub format_version: String,
    pub filename: String,
    pub filepath: String,
    pub entries: Vec<NirsEntry>,
}

pub struct NirsEntry {
    pub metadata: Vec<MetadataTag>,
    pub data_blocks: Vec<DataBlock>,
    pub probe: Probe,
    pub events: Vec<Event>,
    pub auxiliaries: Vec<Auxiliary>,
}

pub struct MetadataTag {
    pub name: String,
    pub value: String,
}

pub struct DataBlock {
    pub time: Vec<f64>,
    pub measurements: Vec<Measurement>,
}

pub struct Measurement {
    pub source_index: usize,          // 1-based (SNIRF spec)
    pub detector_index: usize,        // 1-based
    pub wavelength_index: Option<usize>, // 1-based, None for processed
    pub data_type: i32,               // 1 = CW, 99999 = processed
    pub data_type_label: String,      // "HbO", "HbR", etc.
    pub data: Vec<f64>,               // timeseries for this column
}

pub struct Probe {
    pub wavelengths: Vec<f64>,
    pub sources: Vec<Optode>,
    pub detectors: Vec<Optode>,
    pub landmarks: Vec<Landmark>,
}

pub struct Optode {
    pub id: usize,                    // 0-based internal
    pub name: String,                 // "S1", "D3"
    pub pos_2d: Vector2<f64>,
    pub pos_3d: Vector3<f64>,
}

pub struct Landmark {
    pub label: String,
    pub pos_2d: Option<[f64; 2]>,
    pub pos_3d: Option<[f64; 3]>,
}

pub struct EventMarker {
    pub onset: f64,
    pub duration: f64,
    pub value: f64,
}

pub struct Event {
    pub name: String,
    pub markers: Vec<EventMarker>,
}

pub struct Auxiliary {
    pub name: String,
    pub unit: String,
    pub data: Vec<f64>,
    pub time: Vec<f64>,
}
```

### domain/channel.rs

This is where the measurement→channel grouping lives. Purely derived from
domain types. The key design decision: `ChannelView` carries its own `id`
AND we enforce that `id == position in vec` via a newtype constructor.

```rust
use crate::domain::snirf::{DataBlock, Measurement, NirsEntry, Probe};
use std::collections::BTreeMap;

/// A logical fNIRS channel = one unique (source, detector) pair.
/// `id` is guaranteed to equal this channel's index in the parent Vec.
#[derive(Debug, Clone)]
pub struct ChannelView {
    id: usize,                        // private — always == vec index
    pub name: String,                 // "S1-D2"
    pub source_index: usize,         // 1-based (from SNIRF)
    pub detector_index: usize,       // 1-based
    pub measurement_indices: Vec<usize>, // indices into DataBlock.measurements
}

impl ChannelView {
    pub fn id(&self) -> usize { self.id }

    /// 0-based index into Probe.sources
    pub fn source_0based(&self) -> Option<usize> {
        self.source_index.checked_sub(1)
    }

    /// 0-based index into Probe.detectors
    pub fn detector_0based(&self) -> Option<usize> {
        self.detector_index.checked_sub(1)
    }
}

/// Grouped channel views for one data block.
/// Invariant: channels[i].id() == i for all i.
pub struct ChannelIndex {
    channels: Vec<ChannelView>,
}

impl ChannelIndex {
    pub fn build(block: &DataBlock) -> Self {
        let mut groups: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        for (i, m) in block.measurements.iter().enumerate() {
            groups.entry((m.source_index, m.detector_index))
                .or_default()
                .push(i);
        }

        let channels = groups.into_iter()
            .enumerate()
            .map(|(id, ((src, det), meas))| ChannelView {
                id,
                name: format!("S{}-D{}", src, det),
                source_index: src,
                detector_index: det,
                measurement_indices: meas,
            })
            .collect();

        ChannelIndex { channels }
    }

    pub fn get(&self, id: usize) -> Option<&ChannelView> {
        self.channels.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChannelView> {
        self.channels.iter()
    }

    pub fn len(&self) -> usize { self.channels.len() }
    pub fn is_empty(&self) -> bool { self.channels.is_empty() }

    pub fn as_slice(&self) -> &[ChannelView] { &self.channels }
}

/// What kind of data a block contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataKind {
    RawCW,
    OpticalDensity,
    ProcessedHemoglobin,
    Empty,
}

impl DataKind {
    pub fn detect(block: &DataBlock) -> Self {
        match block.measurements.first() {
            None => DataKind::Empty,
            Some(m) => match m.data_type {
                99999 if m.wavelength_index.is_some() => DataKind::OpticalDensity,
                99999 => DataKind::ProcessedHemoglobin,
                _ => DataKind::RawCW,
            },
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DataKind::RawCW => "raw_cw",
            DataKind::OpticalDensity => "optical_density",
            DataKind::ProcessedHemoglobin => "processed_hemoglobin",
            DataKind::Empty => "empty",
        }
    }
}

/// Which hemoglobin type a measurement represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HemoType { HbO, HbR, HbT, Other }

/// Classify a measurement's signal type.
pub fn classify_signal(m: &Measurement, probe: &Probe) -> SignalKind {
    if m.data_type == 99999 {
        let hemo = match m.data_type_label.to_lowercase().as_str() {
            "hbo" | "dod hbo" => HemoType::HbO,
            "hbr" | "dod hbr" => HemoType::HbR,
            "hbt" => HemoType::HbT,
            _ => HemoType::Other,
        };
        SignalKind::Hemoglobin(hemo)
    } else {
        let wl = m.wavelength_index
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| probe.wavelengths.get(i))
            .copied()
            .unwrap_or(0.0);
        SignalKind::RawAtWavelength(wl)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalKind {
    RawAtWavelength(f64),
    Hemoglobin(HemoType),
}
```

### domain/summary.rs

```rust
use serde::Serialize;
use crate::domain::channel::{ChannelIndex, DataKind};
use crate::domain::snirf::Snirf;

#[derive(Serialize, Clone)]
pub struct BlockSummary {
    pub index: usize,
    pub data_kind: String,
    pub channels: usize,
    pub timepoints: usize,
    pub sampling_rate: f64,
    pub duration: f64,
}

#[derive(Serialize, Clone)]
pub struct SnirfSummary {
    pub filename: String,
    pub format_version: String,
    pub channels: usize,
    pub sources: usize,
    pub detectors: usize,
    pub wavelengths: Vec<f64>,
    pub data_blocks: Vec<BlockSummary>,
    pub event_count: usize,
    pub aux_count: usize,
}

/// Pure function: Snirf → SnirfSummary. No state, no side effects.
pub fn summarize(snirf: &Snirf) -> SnirfSummary {
    let entry = &snirf.entries[0];

    let blocks: Vec<BlockSummary> = entry.data_blocks.iter()
        .enumerate()
        .map(|(i, block)| {
            let ci = ChannelIndex::build(block);
            let sr = if block.time.len() >= 2 {
                1.0 / (block.time[1] - block.time[0])
            } else { 0.0 };
            BlockSummary {
                index: i,
                data_kind: DataKind::detect(block).as_str().to_string(),
                channels: ci.len(),
                timepoints: block.time.len(),
                sampling_rate: sr,
                duration: block.time.last().copied().unwrap_or(0.0),
            }
        })
        .collect();

    let first = blocks.first();
    SnirfSummary {
        filename: snirf.filename.clone(),
        format_version: snirf.format_version.clone(),
        channels: first.map(|b| b.channels).unwrap_or(0),
        sources: entry.probe.sources.len(),
        detectors: entry.probe.detectors.len(),
        wavelengths: entry.probe.wavelengths.clone(),
        data_blocks: blocks,
        event_count: entry.events.len(),
        aux_count: entry.auxiliaries.len(),
    }
}
```

### domain/error.rs

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("No SNIRF file loaded")]
    NoData,
    #[error("No NIRS entries in file")]
    NoEntries,
    #[error("Block index {0} out of range")]
    BlockOutOfRange(usize),
    #[error("Channel {0} not found")]
    ChannelNotFound(usize),
    #[error("{0}")]
    Parse(#[from] anyhow::Error),
    #[error("State lock poisoned")]
    LockPoisoned,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
```

---

## Layer 2: dsp/

Takes `&[f64]` + numeric parameters. Returns `Vec<f64>` or result structs.
Imports nothing from the crate. Could be its own published crate.

### dsp/window.rs

```rust
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowType {
    Hann,
    Hamming,
    Blackman,
}

pub fn apply_window(data: &[f64], window: WindowType) -> Vec<f64> {
    let n = data.len() as f64;
    data.iter().enumerate().map(|(i, &x)| {
        let w = match window {
            WindowType::Hann    => 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1.0)).cos()),
            WindowType::Hamming => 0.54 - 0.46 * (2.0 * PI * i as f64 / (n - 1.0)).cos(),
            WindowType::Blackman => {
                0.42 - 0.5 * (2.0 * PI * i as f64 / (n - 1.0)).cos()
                     + 0.08 * (4.0 * PI * i as f64 / (n - 1.0)).cos()
            }
        };
        x * w
    }).collect()
}
```

### dsp/spectrum.rs

```rust
use realfft::RealFftPlanner;
use crate::dsp::window::{apply_window, WindowType};

pub struct SpectrumResult {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
}

pub fn fft_magnitude(signal: &[f64], sample_rate: f64, window: WindowType) -> SpectrumResult {
    let n = signal.len();
    let windowed = apply_window(signal, window);

    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);
    let mut input = windowed;
    let mut output = fft.make_output_vec();
    fft.process(&mut input, &mut output).unwrap();

    let magnitudes: Vec<f64> = output.iter()
        .map(|c| (c.re * c.re + c.im * c.im).sqrt() * 2.0 / n as f64)
        .collect();

    let df = sample_rate / n as f64;
    let frequencies: Vec<f64> = (0..magnitudes.len()).map(|i| i as f64 * df).collect();

    SpectrumResult { frequencies, magnitudes }
}

pub fn welch_psd(
    signal: &[f64],
    sample_rate: f64,
    segment_len: usize,
    overlap: usize,
    window: WindowType,
) -> SpectrumResult {
    let hop = segment_len - overlap;
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(segment_len);

    let n_bins = segment_len / 2 + 1;
    let n_segments = (signal.len().saturating_sub(overlap)) / hop;
    let mut psd = vec![0.0f64; n_bins];

    for i in 0..n_segments {
        let start = i * hop;
        let seg = &signal[start..start + segment_len];
        let windowed = apply_window(seg, window);
        let mut input = windowed;
        let mut output = fft.make_output_vec();
        fft.process(&mut input, &mut output).unwrap();

        for (j, c) in output.iter().enumerate() {
            psd[j] += (c.re * c.re + c.im * c.im) / (segment_len as f64 * sample_rate);
        }
    }

    if n_segments > 0 {
        psd.iter_mut().for_each(|v| *v /= n_segments as f64);
    }

    let df = sample_rate / segment_len as f64;
    let frequencies: Vec<f64> = (0..n_bins).map(|i| i as f64 * df).collect();

    SpectrumResult { frequencies, magnitudes: psd }
}
```

### dsp/spectrogram.rs

```rust
use realfft::RealFftPlanner;
use crate::dsp::window::{apply_window, WindowType};

pub struct SpectrogramResult {
    pub times: Vec<f64>,
    pub frequencies: Vec<f64>,
    pub power: Vec<Vec<f64>>,   // power[time_frame][freq_bin]
}

pub fn stft(
    signal: &[f64],
    sample_rate: f64,
    window_size: usize,
    hop_size: usize,
    window: WindowType,
) -> SpectrogramResult {
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(window_size);
    let n_bins = window_size / 2 + 1;
    let n_frames = signal.len().saturating_sub(window_size) / hop_size + 1;

    let mut power = Vec::with_capacity(n_frames);
    let mut times = Vec::with_capacity(n_frames);

    for i in 0..n_frames {
        let start = i * hop_size;
        let seg = &signal[start..start + window_size];
        let windowed = apply_window(seg, window);
        let mut input = windowed;
        let mut output = fft.make_output_vec();
        fft.process(&mut input, &mut output).unwrap();

        let frame: Vec<f64> = output.iter()
            .map(|c| {
                let mag_sq = c.re * c.re + c.im * c.im;
                10.0 * mag_sq.max(1e-20).log10()
            })
            .collect();

        power.push(frame);
        times.push((start + window_size / 2) as f64 / sample_rate);
    }

    let df = sample_rate / window_size as f64;
    let frequencies: Vec<f64> = (0..n_bins).map(|i| i as f64 * df).collect();

    SpectrogramResult { times, frequencies, power }
}
```

---

## Layer 3: io/

Imports `domain/` to know what to produce. No Tauri, no services.

### io/snirf_parser.rs

```rust
use anyhow::{Context, Result};
use hdf5::File;
use crate::domain::snirf::*;

pub fn parse_snirf(path: &str) -> Result<Snirf> {
    let file = File::open(path).context("Failed to open HDF5 file")?;
    // ... same parsing logic, returns domain::Snirf
    // NEVER imports commands:: or services::
}
```

---

## Layer 4: state/

Each struct is registered with `.manage()` independently.
Commands request only the state types they need.

### state/session.rs

```rust
use std::sync::RwLock;
use crate::domain::snirf::Snirf;
use crate::domain::channel::ChannelIndex;

/// The loaded SNIRF file + pre-computed channel indices.
/// Registered as a single Tauri managed resource.
pub struct SessionState {
    inner: RwLock<SessionInner>,
}

struct SessionInner {
    snirf: Option<Snirf>,
    /// One ChannelIndex per data block, computed at load time.
    channel_indices: Vec<ChannelIndex>,
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState {
            inner: RwLock::new(SessionInner {
                snirf: None,
                channel_indices: Vec::new(),
            }),
        }
    }
}

impl SessionState {
    /// Load a SNIRF and pre-compute channel indices.
    pub fn load(&self, snirf: Snirf) {
        let indices = snirf.entries.first()
            .map(|e| e.data_blocks.iter().map(ChannelIndex::build).collect())
            .unwrap_or_default();
        let mut inner = self.inner.write().unwrap();
        inner.snirf = Some(snirf);
        inner.channel_indices = indices;
    }

    pub fn unload(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.snirf = None;
        inner.channel_indices.clear();
    }

    /// Read access. Returns None if no file loaded.
    /// The caller holds the RwLock guard for the duration.
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, SessionInner> {
        self.inner.read().unwrap()
    }
}

impl SessionInner {
    pub fn snirf(&self) -> Option<&Snirf> { self.snirf.as_ref() }
    pub fn channels_at(&self, block: usize) -> Option<&ChannelIndex> {
        self.channel_indices.get(block)
    }
    pub fn entry(&self) -> Option<&crate::domain::snirf::NirsEntry> {
        self.snirf.as_ref()?.entries.first()
    }
}
```

### state/selection.rs

```rust
use std::sync::RwLock;

/// User's current selection state.
/// Registered as a separate Tauri managed resource.
pub struct SelectionState {
    inner: RwLock<SelectionInner>,
}

#[derive(Default)]
pub struct SelectionInner {
    pub selected_channels: Vec<usize>,
    pub active_block: usize,
}

impl Default for SelectionState {
    fn default() -> Self {
        SelectionState { inner: RwLock::new(SelectionInner::default()) }
    }
}

impl SelectionState {
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, SelectionInner> {
        self.inner.read().unwrap()
    }
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, SelectionInner> {
        self.inner.write().unwrap()
    }
}
```

---

## Layer 5: services/

Pure functions. Take domain types + dsp functions as inputs.
Return result types. No Tauri, no State, no Arc.

### services/timeseries_service.rs

```rust
use crate::domain::snirf::{NirsEntry, DataBlock};
use crate::domain::channel::{ChannelIndex, ChannelView, DataKind, classify_signal, SignalKind, HemoType};
use serde::Serialize;

#[derive(Serialize)]
pub struct ChannelPayload {
    pub id: usize,
    pub name: String,
    pub series_a: Vec<f64>,
    pub series_b: Vec<f64>,
    pub series_a_label: String,
    pub series_b_label: String,
}

#[derive(Serialize)]
pub struct TimeseriesResult {
    pub time: Vec<f64>,
    pub data_kind: String,
    pub channels: Vec<ChannelPayload>,
}

/// Pure function: entry + block index + channel index → timeseries payload.
/// No Tauri, no state, fully testable.
pub fn build_timeseries(
    entry: &NirsEntry,
    block_idx: usize,
    channels: &ChannelIndex,
) -> Option<TimeseriesResult> {
    let block = entry.data_blocks.get(block_idx)?;
    let data_kind = DataKind::detect(block);
    let time = block.time.clone();

    let channel_payloads: Vec<ChannelPayload> = channels.iter()
        .map(|ch| build_channel_payload(ch, block, &entry.probe, data_kind))
        .collect();

    Some(TimeseriesResult {
        time,
        data_kind: data_kind.as_str().to_string(),
        channels: channel_payloads,
    })
}

fn build_channel_payload(
    ch: &ChannelView,
    block: &DataBlock,
    probe: &crate::domain::snirf::Probe,
    data_kind: DataKind,
) -> ChannelPayload {
    match data_kind {
        DataKind::ProcessedHemoglobin => {
            let (hbo, hbr) = find_hbo_hbr(ch, block, probe);
            ChannelPayload {
                id: ch.id(), name: ch.name.clone(),
                series_a: hbo, series_b: hbr,
                series_a_label: "HbO".into(), series_b_label: "HbR".into(),
            }
        }
        DataKind::RawCW | DataKind::OpticalDensity => {
            let (a, a_wl, b, b_wl) = find_wavelength_pair(ch, block, probe);
            let prefix = if data_kind == DataKind::OpticalDensity { "dOD " } else { "" };
            ChannelPayload {
                id: ch.id(), name: ch.name.clone(),
                series_a: a, series_b: b,
                series_a_label: format!("{prefix}{a_wl:.0} nm"),
                series_b_label: format!("{prefix}{b_wl:.0} nm"),
            }
        }
        DataKind::Empty => ChannelPayload {
            id: ch.id(), name: ch.name.clone(),
            series_a: vec![], series_b: vec![],
            series_a_label: String::new(), series_b_label: String::new(),
        },
    }
}

fn find_hbo_hbr(ch: &ChannelView, block: &DataBlock, probe: &crate::domain::snirf::Probe) -> (Vec<f64>, Vec<f64>) {
    let mut hbo = vec![];
    let mut hbr = vec![];
    for &idx in &ch.measurement_indices {
        let m = &block.measurements[idx];
        match classify_signal(m, probe) {
            SignalKind::Hemoglobin(HemoType::HbO) => hbo = m.data.clone(),
            SignalKind::Hemoglobin(HemoType::HbR) => hbr = m.data.clone(),
            _ => {}
        }
    }
    (hbo, hbr)
}

fn find_wavelength_pair(
    ch: &ChannelView, block: &DataBlock, probe: &crate::domain::snirf::Probe,
) -> (Vec<f64>, f64, Vec<f64>, f64) {
    // Get first two measurements, sort by wavelength (longer = series_a)
    let get_wl = |idx: usize| -> f64 {
        block.measurements.get(idx)
            .and_then(|m| m.wavelength_index)
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| probe.wavelengths.get(i))
            .copied()
            .unwrap_or(0.0)
    };

    let m0 = ch.measurement_indices.first().copied().unwrap_or(0);
    let m1 = ch.measurement_indices.get(1).copied().unwrap_or(0);
    let wl0 = get_wl(m0);
    let wl1 = get_wl(m1);
    let d0 = block.measurements.get(m0).map(|m| m.data.clone()).unwrap_or_default();
    let d1 = block.measurements.get(m1).map(|m| m.data.clone()).unwrap_or_default();

    if wl0 >= wl1 { (d0, wl0, d1, wl1) } else { (d1, wl1, d0, wl0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::snirf::*;

    fn make_entry(n_samples: usize) -> NirsEntry {
        NirsEntry {
            metadata: vec![],
            data_blocks: vec![DataBlock {
                time: (0..n_samples).map(|i| i as f64 * 0.1).collect(),
                measurements: vec![
                    Measurement {
                        source_index: 1, detector_index: 1,
                        wavelength_index: Some(1), data_type: 1,
                        data_type_label: String::new(),
                        data: vec![1.0; n_samples],
                    },
                    Measurement {
                        source_index: 1, detector_index: 1,
                        wavelength_index: Some(2), data_type: 1,
                        data_type_label: String::new(),
                        data: vec![2.0; n_samples],
                    },
                ],
            }],
            probe: Probe {
                wavelengths: vec![760.0, 850.0],
                sources: vec![], detectors: vec![], landmarks: vec![],
            },
            events: vec![], auxiliaries: vec![],
        }
    }

    #[test]
    fn builds_raw_channel_with_correct_wavelength_order() {
        let entry = make_entry(100);
        let ci = ChannelIndex::build(&entry.data_blocks[0]);
        let result = build_timeseries(&entry, 0, &ci).unwrap();
        assert_eq!(result.channels.len(), 1);
        assert_eq!(result.channels[0].series_a_label, "850 nm"); // longer = a
        assert_eq!(result.channels[0].series_b_label, "760 nm");
    }
}
```

### services/spectral_service.rs

```rust
use crate::domain::snirf::NirsEntry;
use crate::domain::channel::ChannelIndex;
use crate::dsp::spectrum::{fft_magnitude, welch_psd, SpectrumResult};
use crate::dsp::window::WindowType;
use serde::Serialize;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpectrumMethod {
    Fft,
    Psd,
}

#[derive(Serialize)]
pub struct ChannelSpectrum {
    pub channel_id: usize,
    pub channel_name: String,
    pub label: String,
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
}

/// Pure function: compute spectra for selected channels.
pub fn compute_spectra(
    entry: &NirsEntry,
    block_idx: usize,
    channels: &ChannelIndex,
    selected: &[usize],
    method: SpectrumMethod,
    window: WindowType,
) -> Vec<ChannelSpectrum> {
    let block = match entry.data_blocks.get(block_idx) {
        Some(b) => b,
        None => return vec![],
    };

    let sample_rate = if block.time.len() >= 2 {
        1.0 / (block.time[1] - block.time[0])
    } else { return vec![]; };

    let mut out = Vec::new();
    for &ch_id in selected {
        let Some(ch) = channels.get(ch_id) else { continue };

        for (pos, &meas_idx) in ch.measurement_indices.iter().enumerate() {
            let m = &block.measurements[meas_idx];

            let label = if m.data_type_label.is_empty() {
                entry.probe.wavelengths.get(pos)
                    .map(|wl| format!("{wl:.0}nm"))
                    .unwrap_or_else(|| format!("series{pos}"))
            } else {
                m.data_type_label.clone()
            };

            let result = match method {
                SpectrumMethod::Fft => fft_magnitude(&m.data, sample_rate, window),
                SpectrumMethod::Psd => {
                    let seg = 256.min(m.data.len()).max(4);
                    if m.data.len() >= seg {
                        welch_psd(&m.data, sample_rate, seg, seg / 2, window)
                    } else {
                        fft_magnitude(&m.data, sample_rate, window)
                    }
                }
            };

            out.push(ChannelSpectrum {
                channel_id: ch.id(),
                channel_name: ch.name.clone(),
                label,
                frequencies: result.frequencies,
                magnitudes: result.magnitudes,
            });
        }
    }
    out
}
```

---

## Layer 6: commands/

Paper-thin. Each one: read state → call service → return DTO.

### commands/file_commands.rs

```rust
use tauri::{Emitter, State};
use crate::state::session::SessionState;
use crate::state::selection::SelectionState;
use crate::domain::summary::summarize;
use crate::domain::error::AppError;
use crate::io::snirf_parser::parse_snirf;

#[tauri::command]
pub fn import_snirf(
    path: String,
    session: State<SessionState>,
    app: tauri::AppHandle,
) -> Result<crate::domain::summary::SnirfSummary, AppError> {
    let snirf = parse_snirf(&path).map_err(AppError::Parse)?;
    let summary = summarize(&snirf);
    session.load(snirf);
    let _ = app.emit("snirf-loaded", summary.clone());
    Ok(summary)
}
```

### commands/timeseries_commands.rs

```rust
use tauri::State;
use crate::state::session::SessionState;
use crate::state::selection::SelectionState;
use crate::services::timeseries_service;

#[tauri::command]
pub fn get_timeseries_data(
    session: State<SessionState>,
    selection: State<SelectionState>,
) -> Option<timeseries_service::TimeseriesResult> {
    let sess = session.read();
    let sel = selection.read();
    let entry = sess.entry()?;
    let block_idx = sel.active_block;
    let channels = sess.channels_at(block_idx)?;
    timeseries_service::build_timeseries(entry, block_idx, channels)
}
```

### commands/spectral_commands.rs

```rust
use tauri::State;
use crate::state::session::SessionState;
use crate::state::selection::SelectionState;
use crate::services::spectral_service::{self, SpectrumMethod, ChannelSpectrum};
use crate::dsp::window::WindowType;

#[tauri::command]
pub fn get_spectrum(
    method: Option<SpectrumMethod>,
    window: Option<WindowType>,
    session: State<SessionState>,
    selection: State<SelectionState>,
) -> Vec<ChannelSpectrum> {
    let sess = session.read();
    let sel = selection.read();

    let Some(entry) = sess.entry() else { return vec![] };
    let Some(channels) = sess.channels_at(sel.active_block) else { return vec![] };

    let selected: Vec<usize> = if sel.selected_channels.is_empty() {
        (0..channels.len()).collect()
    } else {
        sel.selected_channels.clone()
    };

    spectral_service::compute_spectra(
        entry,
        sel.active_block,
        channels,
        &selected,
        method.unwrap_or(SpectrumMethod::Fft),
        window.unwrap_or(WindowType::Hann),
    )
}
```

### commands/selection_commands.rs

```rust
use tauri::{Emitter, State};
use crate::state::selection::SelectionState;

#[tauri::command]
pub fn set_selected_channels(
    channel_ids: Vec<usize>,
    selection: State<SelectionState>,
    app: tauri::AppHandle,
) {
    selection.write().selected_channels = channel_ids.clone();
    let _ = app.emit("channels-selected", serde_json::json!({ "channel_ids": channel_ids }));
}

#[tauri::command]
pub fn set_active_block(
    index: usize,
    selection: State<SelectionState>,
    app: tauri::AppHandle,
) {
    selection.write().active_block = index;
    let _ = app.emit("block-changed", index);
}
```

---

## lib.rs — the wiring

```rust
pub mod domain;
pub mod dsp;
pub mod io;
pub mod state;
pub mod services;
pub mod commands;

use state::session::SessionState;
use state::selection::SelectionState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(SessionState::default())
        .manage(SelectionState::default())
        .invoke_handler(tauri::generate_handler![
            commands::file_commands::import_snirf,
            commands::info_commands::get_summary,
            commands::probe_commands::get_probe_layout,
            commands::selection_commands::set_selected_channels,
            commands::selection_commands::set_active_block,
            commands::timeseries_commands::get_timeseries_data,
            commands::spectral_commands::get_spectrum,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Why this separation matters

### Testing tiers

| Tier | What you test | What you need |
|------|--------------|---------------|
| `dsp/` | FFT peak at known frequency | Just construct a sine wave `Vec<f64>` |
| `domain/` | ChannelIndex groups correctly | Just construct a `DataBlock` |
| `services/` | Timeseries builds correct payloads | Construct a `NirsEntry` + `ChannelIndex` |
| `commands/` | Integration only | Need Tauri test harness (rare) |

90% of your logic lives in tiers 1-3. None of them need Tauri.

### Adding a new feature

Say you want to add bandpass filtering. The path is:

1. `dsp/filter.rs` — `pub fn bandpass(signal: &[f64], low: f64, high: f64, sample_rate: f64) -> Vec<f64>`
2. `services/filter_service.rs` — iterates channels, calls `dsp::filter::bandpass`, returns filtered data
3. `commands/filter_commands.rs` — reads SessionState + SelectionState, calls service, returns JSON
4. Register in `lib.rs`

Each step touches exactly one layer. No existing code changes.

### What does NOT belong anywhere

- `println!` debugging → use `log::info!` / `log::debug!`
- Display/format impls → keep in `domain/`, but don't put them in the same
  file as the data types. Use a separate `domain/display.rs` if you want them.
- Test SNIRF construction helpers → `domain/test_fixtures.rs` behind `#[cfg(test)]`
