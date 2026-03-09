use serde::Serialize;

use crate::domain::nirs_view::NirsView;
use crate::domain::snirf::SNIRF;

#[derive(Serialize, Clone)]
pub struct EventSummary {
    pub name: String,
    pub marker_count: usize,
}
#[derive(Serialize, Clone)]
pub struct WavelengthInfo {
    pub wavelengths_nm: Vec<f64>,
}

#[derive(Serialize, Clone)]
pub struct SnirfSummary {
    pub filename: String,
    pub format_version: String,
    pub data_kind: String, // "raw_cw" | "processed_hemoglobin" | "empty"
    pub channels: usize,
    pub sources: usize,
    pub detectors: usize,
    pub timepoints: usize,
    pub sampling_rate: f64,
    pub duration: f64,
    pub wavelengths: Vec<f64>,
    pub events: Vec<EventSummary>,
    pub aux_count: usize,
}

impl SnirfSummary {
    pub fn from_snirf(snirf: &SNIRF) -> Self {
        let entry = &snirf.nirs_entries[0];
        let view = NirsView::new(entry);

        let data_kind = match view.data_kind() {
            crate::domain::nirs_view::DataKind::RawCW => "raw_cw",
            crate::domain::nirs_view::DataKind::ProcessedHemoglobin => "processed hemoglobin",
            crate::domain::nirs_view::DataKind::Empty => "empty",
        };

        SnirfSummary {
            filename: snirf.file_descriptor.filename.clone(),
            format_version: snirf.format_version.clone(),
            data_kind: data_kind.to_string(),
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
        }
    }
}
