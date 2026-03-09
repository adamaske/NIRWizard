use nalgebra::{Vector2, Vector3};
use std::fmt;

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

impl fmt::Display for SNIRF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "SNIRF  {}  (v{})",
            self.file_descriptor.filename, self.format_version
        )?;
        writeln!(f, "  Path: {}", self.file_descriptor.filepath)?;
        writeln!(f, "  Nirs entries: {}", self.nirs_entries.len())?;
        for (i, entry) in self.nirs_entries.iter().enumerate() {
            if self.nirs_entries.len() > 1 {
                writeln!(f, "  ── nirs {} ──", i + 1)?;
            }
            write!(f, "{}", entry)?;
        }
        Ok(())
    }
}

impl fmt::Display for NirsEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Metadata
        writeln!(f, "  Metadata ({} tags):", self.metadata.len())?;
        for tag in &self.metadata {
            writeln!(f, "    {:<28} {}", tag.name, tag.value)?;
        }

        // Probe
        write!(f, "{}", self.probe)?;

        // Data blocks
        writeln!(f, "  Data blocks: {}", self.data_blocks.len())?;
        for (i, block) in self.data_blocks.iter().enumerate() {
            let n_channels = block.unique_channel_pairs().len();
            let n_measurements = block.measurements.len();
            let n_timepoints = block.time.len();

            let (duration, sr) = if block.time.len() >= 2 {
                let dur = block.time.last().copied().unwrap_or(0.0);
                let sr = 1.0 / (block.time[1] - block.time[0]);
                (dur, sr)
            } else {
                (0.0, 0.0)
            };

            writeln!(
                f,
                "    data{}: {} channels, {} measurements, {} timepoints, {:.1}s @ {:.2} Hz",
                i + 1,
                n_channels,
                n_measurements,
                n_timepoints,
                duration,
                sr,
            )?;

            // Show data types present
            let mut type_counts: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for m in &block.measurements {
                let label = if m.data_type == 99999 {
                    if m.data_type_label.is_empty() {
                        "processed".to_string()
                    } else {
                        m.data_type_label.clone()
                    }
                } else {
                    format!("dataType={}", m.data_type)
                };
                *type_counts.entry(label).or_default() += 1;
            }
            let types_str: Vec<String> = type_counts
                .iter()
                .map(|(k, v)| format!("{}×{}", v, k))
                .collect();
            writeln!(f, "      measurements: [{}]", types_str.join(", "))?;
        }

        // Events
        if self.events.is_empty() {
            writeln!(f, "  Events: none")?;
        } else {
            writeln!(f, "  Events ({}):", self.events.len())?;
            for ev in &self.events {
                writeln!(f, "    {:<28} {} markers", ev.name, ev.markers.len())?;
            }
        }

        // Auxiliaries
        if self.auxiliaries.is_empty() {
            writeln!(f, "  Auxiliaries: none")?;
        } else {
            writeln!(f, "  Auxiliaries ({}):", self.auxiliaries.len())?;
            for aux in &self.auxiliaries {
                writeln!(
                    f,
                    "    {:<28} {} samples  ({})",
                    aux.name,
                    aux.data.len(),
                    aux.unit,
                )?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wl_str: Vec<String> = self
            .wavelengths
            .iter()
            .map(|w| format!("{:.0}", w))
            .collect();
        writeln!(f, "  Probe:")?;
        writeln!(f, "    Wavelengths: [{}] nm", wl_str.join(", "))?;
        writeln!(f, "    Sources:     {}", self.sources.len())?;
        writeln!(f, "    Detectors:   {}", self.detectors.len())?;

        if let Some(ref cs) = self.coordinate_system {
            writeln!(f, "    Coord system: {}", cs)?;
        }
        if let Some(ref lm) = self.landmarks {
            let names: Vec<&str> = lm.iter().map(|l| l.label.as_str()).collect();
            writeln!(f, "    Landmarks ({}): {}", lm.len(), names.join(", "))?;
        }
        Ok(())
    }
}
