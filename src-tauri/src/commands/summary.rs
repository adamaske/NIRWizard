use serde::Serialize;

use crate::domain::snirf::SNIRF;

#[derive(Serialize, Clone)]
pub struct EventSummary {
    pub name: String,
    pub marker_count: usize,
}

#[derive(Serialize, Clone)]
pub struct SnirfSummary {
    pub filename: String,
    pub channels: usize,
    pub sources: usize,
    pub detectors: usize,
    pub timepoints: usize,
    pub sampling_rate: f64,
    pub duration: f64,
    pub hbo_wavelength: usize,
    pub hbr_wavelength: usize,
    pub events: Vec<EventSummary>,
    pub aux_count: usize,
}

impl SnirfSummary {
    pub fn from_snirf(snirf: &SNIRF) -> Self {
        let time = &snirf.nirs_entries[0].data_blocks[0].time;
        let sampling_rate = if time.len() >= 2 {
            1.0 / (time[1] - time[0])
        } else {
            0.0
        };

        SnirfSummary {
            filename: snirf.file_descriptor.filename.clone(),
            channels: snirf.channels.channels.len(),
            sources: snirf.nirs_entries[0].probe.sources.len(),
            detectors: snirf.nirs_entries[0].probe.detectors.len(),
            timepoints: time.len(),
            sampling_rate,
            duration: time.last().copied().unwrap_or(0.0),
            hbo_wavelength: snirf.wavelengths.hbo_wl,
            hbr_wavelength: snirf.wavelengths.hbr_wl,
            events: snirf
                .events
                .events
                .iter()
                .map(|e| EventSummary {
                    name: e.name.clone(),
                    marker_count: e.markers.len(),
                })
                .collect(),
            aux_count: snirf.biosignals.auxilaries.len(),
        }
    }
}
