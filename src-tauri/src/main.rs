mod commands;
mod domain;
mod io;
mod processing;
mod state;

use crate::io::snirf_parser::parse_snirf;
use crate::state::AppState;
use tauri::{Emitter, Manager};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            // Set NIRWIZARD_DEFAULT_SNIRF=/path/to/file.snirf to auto-load on startup
            if let Ok(default_path) = std::env::var("NIRWIZARD_DEFAULT_SNIRF") {
                if let Ok(snirf) = parse_snirf(&default_path) {
                    let summary = commands::SnirfSummary::from_snirf(&snirf);
                    let state = app.state::<AppState>();
                    if let Ok(mut session) = state.session.write() {
                        session.snirf = Some(snirf);
                    }
                    let _ = app.emit("snirf-loaded", summary);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_snirf,
            commands::get_snirf_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
