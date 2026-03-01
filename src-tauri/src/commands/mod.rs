mod summary;
pub mod timeseries;

pub use summary::{EventSummary, SnirfSummary};
pub use timeseries::get_timeseries_data;

use tauri::Emitter;

use crate::io::snirf_parser::parse_snirf;
use crate::state::AppState;

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

    state.session.write().map_err(|e| e.to_string())?.snirf = Some(snirf);

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
