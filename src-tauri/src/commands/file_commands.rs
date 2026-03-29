use crate::domain::error::{LogErr, NWError};
use crate::domain::summary::SnirfSummary;
use crate::services::session_service::{load_snirf, LoadResult};
use crate::state::session::SessionState;
use tauri::{Emitter, State};

#[tauri::command]
pub fn import_snirf(
    path: String,
    session: State<SessionState>,
    app: tauri::AppHandle,
) -> Result<SnirfSummary, NWError> {
    let result: LoadResult = load_snirf(&path)?;

    session.load(result.snirf, result.channel_indices);

    let _ = app.emit("snirf-loaded", result.summary.clone());
    Ok(result.summary)
}

// TODO : export_snirf
