use tauri::State;

use crate::domain::summary::SnirfSummary;
use crate::state::session::SessionState;
#[tauri::command]
pub fn get_snirf_summary(state: tauri::State<SessionState>) -> Option<SnirfSummary> {
    // TODO : Check if state has a loaded snirf, then summerize it
    Ok(SnirfSummary::from_snirf(state.inner().snirf));
}
