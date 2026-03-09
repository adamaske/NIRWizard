use nalgebra::{Vector2, Vector3};

pub struct SNIRF {
    pub format_version: String,
    pub file_descriptor: FileDescriptor,
    pub nirs_entries: Vec<NirsEntry>,
}

// =========================
// File Descriptor
// =========================
pub struct FileDescriptor {
    pub filepath: String,
    pub filename: String,
}

// =========================
// NIRS
// =========================
pub struct NirsEntry {
    pub metadata: Vec<MetadataTag>,
    pub data_blocks: Vec<DataBlock>,
    pub probe: Probe,
    pub events: Vec<Event>,
    pub auxiliaries: Vec<AuxiliaryData>,
}

// =========================
// Metadata
// =========================
pub struct MetadataTag {
    pub name: String,
    pub value: String,
}

// =========================
// Data
// =========================
pub struct Measurement {
    pub source_index: usize,
    pub detector_index: usize,
    pub wavelength_index: usize,

    pub data_type: i32,          // 1 = CW, 99999=processed
    pub data_type_label: String, // i.e "HbO", "HbR", "OD Hbo"
    pub data_type_index: i32,
    pub data_unit: Option<String>,
    pub data: Vec<f64>,

    // OPTIONAL FIELDS
    pub wavelength_actual: Option<f64>,
    pub source_power: Option<f64>,
    pub detector_gain: Option<f64>,
    pub module_index: Option<f64>,
}

pub struct DataBlock {
    pub time: Vec<f64>,
    pub measurements: Vec<Measurement>,
}

impl DataBlock {
    /// Returns one `(source_index, detector_index)` pair per unique fNIRS channel,
    /// in the order they first appear in the measurement list.
    ///
    /// In SNIRF a "channel" is a physical source–detector pair.  Each pair typically
    /// has one measurement entry per wavelength, so the raw `measurements` vec
    /// contains duplicates.  This method strips those duplicates out.
    pub fn unique_channel_pairs(&self) -> Vec<(usize, usize)> {
        // `HashSet::insert` returns `true` the first time a value is inserted
        // and `false` if it was already present — perfect for dedup-in-place.
        let mut seen = std::collections::HashSet::new();

        self.measurements
            .iter()
            // Keep only the first occurrence of each (source, detector) pair.
            // The closure takes `&&Measurement` because `.iter()` yields `&Measurement`
            // and `.filter()` wraps that in another `&`, so we shadow with `m` for clarity.
            .filter(|m| seen.insert((m.source_index, m.detector_index)))
            .map(|m| (m.source_index, m.detector_index))
            .collect()
    }
}

// =========================
// Probe
// =========================
pub struct Probe {
    pub wavelengths: Vec<f64>,
    pub wavelength_emission: Option<Vec<f64>>,
    //
    pub sources: Vec<Optode>,
    pub detectors: Vec<Optode>,
    // Coordinates & Landmarks
    pub landmarks: Option<Vec<Landmark>>,
    pub coordinate_system: Option<String>,
    pub coordinate_system_description: Option<String>,
    pub use_local_index: Option<i32>,
    // TODO : FD, TD, DCS fields
}

pub struct Landmark {
    pub label: String,
    pub pos_2d: Option<[f64; 2]>,
    pub pos_3d: Option<[f64; 3]>,
}

pub struct Optode {
    pub name: String,
    pub id: usize,
    pub pos_3d: Vector3<f64>,
    pub pos_2d: Vector2<f64>,
}

// =========================
// Events / Markers / Triggers
// =========================
pub struct EventMarker {
    pub onset: f64,    // seconds
    pub duration: f64, // seconds
    pub value: f64,
}

pub struct Event {
    pub name: String,
    pub markers: Vec<EventMarker>,
}

// =========================
// Auxiliary Data
// =========================
pub struct AuxiliaryData {
    pub name: String,
    pub unit: String,
    pub data: Vec<f64>,
    pub time: Vec<f64>,
    pub time_offset: Option<f64>,
}
