mod commands;
use commands::info_commands;

mod domain;
mod io;
mod setup;
mod spectral;
mod state;

use crate::state::state_old::AppState;

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
        .manage(AppState::default())
        .setup(setup::setup_app) // Automatically passes AppState ?
        .invoke_handler(tauri::generate_handler![
            commands::import_snirf,
            commands::export_snirf,
            commands::get_snirf_summary,
            // INFO
            info_commands::get_snirf_summary_new,
            //
            commands::timeseries::get_timeseries_data,
            commands::timeseries::set_cursor_timepoint,
            //
            commands::probe::get_probe_layout,
            commands::probe::set_selected_channels,
            commands::probe::set_active_block,
            commands::scene::get_anatomy_geometry,
            //
            commands::scene::set_anatomy_transform,
            commands::scene::set_anatomy_opacity,
            commands::scene::get_optode_layout_3d,
            commands::scene::set_optode_layout_transform,
            commands::scene::set_optode_layout_settings,
            //
            commands::anatomy::load_mri,
            //
            commands::voxel::list_voxel_volumes,
            commands::voxel::get_voxel_volume_info,
            commands::voxel::get_voxel_slice,
            //
            commands::spectral::get_spectrums,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
