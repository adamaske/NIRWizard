mod state;
mod domain;
mod io;

use crate::io::snirf_parser::parse_snirf;
use crate::domain::snirf::SNIRF;
use crate::state::AppState;

use serde::Serialize;
use tauri::{Manager, Emitter};

// Lightweight summary sent to the frontend
#[derive(Serialize, Clone)]
pub struct SnirfSummary {
    pub filename: String,
    pub channels: usize,
    pub sampling_rate: f64,
    pub duration: f64,
}

impl SnirfSummary {
    fn from_snirf(snirf: &SNIRF) -> Self {
        let channels = snirf.channels.channels.len();
        let time = &snirf.channels.time;
        let sampling_rate = if time.len() >= 2 {
            1.0 / (time[1] - time[0])
        } else {
            0.0
        };
        let duration = time.last().copied().unwrap_or(0.0);

        SnirfSummary {
            filename: snirf.fd.name.clone(),
            channels,
            sampling_rate,
            duration,
        }
    }
}

// Parses and stores a SNIRF file, then broadcasts its summary
#[tauri::command]
fn load_snirf(
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

// Returns the summary of whatever is currently loaded (None if nothing)
#[tauri::command]
fn get_snirf_summary(state: tauri::State<AppState>) -> Option<SnirfSummary> {
    let session = state.session.read().ok()?;
    session.snirf.as_ref().map(SnirfSummary::from_snirf)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            // Set NIRWIZARD_DEFAULT_SNIRF=/path/to/file.snirf to auto-load on startup
            if let Ok(default_path) = std::env::var("NIRWIZARD_DEFAULT_SNIRF") {
                if let Ok(snirf) = parse_snirf(&default_path) {
                    let summary = SnirfSummary::from_snirf(&snirf);
                    let state = app.state::<AppState>();
                    if let Ok(mut session) = state.session.write() {
                        session.snirf = Some(snirf);
                    }
                    let _ = app.emit("snirf-loaded", summary);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_snirf, get_snirf_summary])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
