pub mod commands;
pub mod domain;
pub mod dsp;
pub mod io;
pub mod services;
pub mod state;

use state::selection::SelectionState;
use state::session::SessionState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .level_for("tao", log::LevelFilter::Warn)
                .level_for("wry", log::LevelFilter::Warn)
                .level_for("tracing", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .manage(SessionState::default())
        .manage(SelectionState::default())
        .invoke_handler(tauri::generate_handler![
            // File I/O
            commands::file_commands::import_snirf,
            //commands::file_commands::export_snirf,
            // Info
            commands::info_commands::get_snirf_summary,
            // Timeseries
            commands::timeseries_commands::get_timeseries_data,
            commands::timeseries_commands::set_cursor_timepoint,
            // Probe / channel selection
            commands::probe_commands::get_probe_layout,
            commands::selection_commands::set_selected_channels,
            commands::selection_commands::set_active_block,
            // Spectral
            commands::spectral_commands::get_spectrum,
            commands::spectral_commands::get_spectrogram,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
