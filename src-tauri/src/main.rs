mod commands;
mod domain;
mod io;
mod processing;
mod setup;
mod state;

use crate::state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(setup::run)
        .invoke_handler(tauri::generate_handler![
            commands::load_snirf,
            commands::export_snirf,
            commands::get_snirf_summary,
            commands::timeseries::get_timeseries_data,
            commands::timeseries::set_cursor_timepoint,
            commands::probe::get_probe_layout,
            commands::probe::set_selected_channels,
            commands::pipeline::add_pipeline_step,
            commands::pipeline::remove_pipeline_step,
            commands::pipeline::move_pipeline_step,
            commands::pipeline::get_pipeline_summary,
            commands::scene::get_anatomy_geometry,
            commands::scene::set_anatomy_transform,
            commands::scene::set_anatomy_opacity,
            commands::scene::get_optode_layout_3d,
            commands::scene::set_optode_layout_transform,
            commands::scene::set_optode_layout_settings,
            commands::anatomy::load_mri,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
