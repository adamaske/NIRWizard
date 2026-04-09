use tauri::State;

use crate::domain::summary::SnirfSummary;
use crate::state::session::SessionState;

#[tauri::command]
pub fn get_snirf_summary_new(state: tauri::State<SessionState>) -> Option<SnirfSummary> {
    // TODO : Check if we have a loaded snirf
    let session = state.session.read().unwrap();
    session.snirf.as_ref().map(|s| SnirfSummary::from_snirf(s))
}
