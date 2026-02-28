use serde::Serialize;
use tauri::Emitter;

use crate::domain::snirf::SNIRF;
use crate::io::snirf_parser::parse_snirf;
use crate::state::AppState;

// =============================================================================
// IPC DTOs
// =============================================================================

#[derive(Serialize, Clone)]
pub struct EventSummary {
    pub name: String,
    pub marker_count: usize,
}

/// Lightweight summary of a loaded SNIRF file, sent to the frontend.
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
        let time = &snirf.channels.time;
        let sampling_rate = if time.len() >= 2 {
            1.0 / (time[1] - time[0])
        } else {
            0.0
        };

        SnirfSummary {
            filename: snirf.fd.name.clone(),
            channels: snirf.channels.channels.len(),
            sources: snirf.probe.sources.len(),
            detectors: snirf.probe.detectors.len(),
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

// =============================================================================
// Tauri commands
// =============================================================================

/// Parses and stores a SNIRF file, then broadcasts its summary.
#[tauri::command]
pub fn load_snirf(
    path: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<SnirfSummary, String> {
    let snirf = parse_snirf(&path)?;
    let summary = SnirfSummary::from_snirf(&snirf);

    state
        .session
        .write()
        .map_err(|e| e.to_string())?
        .snirf = Some(snirf);

    app.emit("snirf-loaded", summary.clone())
        .map_err(|e| e.to_string())?;

    Ok(summary)
}

/// Returns the summary of whatever is currently loaded (`None` if nothing).
#[tauri::command]
pub fn get_snirf_summary(state: tauri::State<AppState>) -> Option<SnirfSummary> {
    let session = state.session.read().ok()?;
    session.snirf.as_ref().map(SnirfSummary::from_snirf)
}
