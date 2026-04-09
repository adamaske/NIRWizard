use tauri::State;

use crate::domain::summary::SnirfSummary;
use crate::state::session::SessionState;

#[tauri::command]
pub fn get_snirf_summary_new(state: tauri::State<SessionState>) -> Option<SnirfSummary> {
    // TODO : Check if we have a loaded snirf
    Ok(SnirfSummary::from_snirf(state.session.snirf))
}
