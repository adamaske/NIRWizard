use serde::Serialize;

use crate::domain::nirs_view::{DataKind, NirsView};
use crate::domain::snirf::Snirf;

#[derive(Serialize, Clone)]
pub struct EventSummary {
    pub name: String,
    pub marker_count: usize,
}

#[derive(Serialize, Clone)]
pub struct BlockSummary {
    pub index: usize,
    pub data_kind: String, // "raw_cw" | "processed_hemoglobin" | "empty"
    pub channels: usize,
    pub timepoints: usize,
    pub sampling_rate: f64,
    pub duration: f64,
}

#[derive(Serialize, Clone)]
pub struct SnirfSummary {
    pub filename: String,
    pub format_version: String,
    /// Describes the first data block (for quick access).
    pub data_kind: String,
    pub channels: usize,
    pub sources: usize,
    pub detectors: usize,
    pub timepoints: usize,
    pub sampling_rate: f64,
    pub duration: f64,
    pub wavelengths: Vec<f64>,
    pub events: Vec<EventSummary>,
    pub aux_count: usize,
    /// One entry per data block in the SNIRF file.
    pub data_blocks: Vec<BlockSummary>,
}

fn data_kind_str(dk: DataKind) -> &'static str {
    match dk {
        DataKind::RawCW => "raw_cw",
        DataKind::OpticalDensity => "optical_density",
        DataKind::ProcessedHemoglobin => "processed_hemoglobin",
        DataKind::Empty => "empty",
    }
}

impl SnirfSummary {
    pub fn from_snirf(snirf: &Snirf) -> Self {
        let entry = &snirf.nirs_entries[0];
        let view = NirsView::new(entry);

        let data_blocks: Vec<BlockSummary> = (0..view.block_count())
            .map(|i| BlockSummary {
                index: i,
                data_kind: data_kind_str(view.data_kind_at(i)).to_string(),
                channels: view.channels_at(i).len(),
                timepoints: view.time_at(i).len(),
                sampling_rate: view.sampling_rate_at(i),
                duration: view.duration_at(i),
            })
            .collect();

        SnirfSummary {
            filename: snirf.file_descriptor.filename.clone(),
            format_version: snirf.format_version.clone(),
            data_kind: data_kind_str(view.data_kind()).to_string(),
            channels: view.channel_count(),
            sources: entry.probe.sources.len(),
            detectors: entry.probe.detectors.len(),
            timepoints: view.timepoints(),
            sampling_rate: view.sampling_rate(),
            duration: view.duration(),
            wavelengths: entry.probe.wavelengths.clone(),
            events: entry
                .events
                .iter()
                .map(|e| EventSummary {
                    name: e.name.clone(),
                    marker_count: e.markers.len(),
                })
                .collect(),
            aux_count: entry.auxiliaries.len(),
            data_blocks,
        }
    }
}
