use crate::domain::error::{LogErr, NWError};

use crate::domain::summary::SnirfSummary;
use crate::io::snirf_parser::parse_snirf;

use crate::state::session::SessionState;
use tauri::{Emitter, State};

#[tauri::command]
pub fn import_snirf_new(
    path: String,
    session: State<SessionState>,
    app: tauri::AppHandle,
) -> Result<SnirfSummary, NWError> {
    let snirf = parse_snirf(&path)?;
    let summary = SnirfSummary::from_snirf(&snirf);

    // TODO : Cache channel index, time, frequency and time-frequency

    session.load(snirf);

    app.emit("snirf-loaded", summary.clone());
    Ok(summary)
}

#[tauri::command]
pub fn export_snirf_new(
    path: String,
    session: State<SessionState>,
) -> Result<SnirfSummary, NWError> {
    Err(NWError::GenericError(
        "Export function not implemented".to_string(),
    ))
}
// TODO : export_snirf
