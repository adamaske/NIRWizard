pub mod probe;
pub mod scene;
pub mod spectral;
mod summary;
pub mod timeseries;
pub mod voxel;

pub use summary::SnirfSummary;
use tauri::Emitter;

use crate::domain::probe::OptodeLayout;
use crate::io::snirf_exporter;
use crate::io::snirf_parser::parse_snirf;
use crate::state::AppState;
pub mod anatomy;

// =============================================================================
// Tauri commands
// =============================================================================

// TODO : Move the SNIRF import and export commands to a proper file

// Parses and stores a SNIRF file, then broadcasts its summary.
#[tauri::command]
pub fn import_snirf(
    path: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<SnirfSummary, String> {
    let snirf = parse_snirf(&path).map_err(|e| format!("{e:#}"))?;
    let summary = SnirfSummary::from_snirf(&snirf);
    let optode_layout = OptodeLayout::from_snirf(&snirf);

    {
        let mut session = state.nirs.write().map_err(|e| e.to_string())?;
        session.snirf = Some(snirf);
        session.optode_layout = Some(optode_layout);
    }

    app.emit("snirf-loaded", summary.clone())
        .map_err(|e| e.to_string())?;

    Ok(summary)
}

#[tauri::command]
pub fn export_snirf(path: String, state: tauri::State<AppState>) -> Result<SnirfSummary, String> {
    let nirs = state.nirs.read().map_err(|e| e.to_string())?;
    let snirf = nirs.snirf.as_ref().ok_or("No SNIRF data to export")?;

    snirf_exporter::export_snirf(snirf, &path)?;

    Ok(SnirfSummary::from_snirf(snirf))
}

/// Returns the summary of whatever is currently loaded (`None` if nothing).
#[tauri::command]
pub fn get_snirf_summary(state: tauri::State<AppState>) -> Option<SnirfSummary> {
    let nirs = state.nirs.read().ok()?;
    nirs.snirf.as_ref().map(SnirfSummary::from_snirf)
}
