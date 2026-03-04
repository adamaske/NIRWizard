use std::path::PathBuf;

use tauri::{Emitter, Manager};

use crate::commands::SnirfSummary;
use crate::domain::probe::OptodeLayout;
use crate::io::anatomy_importer::load_subject_anatomy;
use crate::io::snirf_parser::parse_snirf;
use crate::state::AppState;

pub fn run(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // NIRWIZARD_DATA_DIRECTORY — load from a conventional directory layout:
    //   {DATA_DIR}/input/nirs/processed.snirf
    //   {DATA_DIR}/input/sub-XXX_T1w.nii.gz
    if let Ok(data_dir) = std::env::var("NIRWIZARD_DATA_DIRECTORY") {
        // Clone what we need to move into the background thread.
        let handle = app.handle().clone();

        std::thread::spawn(move || {
            // Wait for the webview to finish loading before emitting events.
            std::thread::sleep(std::time::Duration::from_millis(500));

            let data_dir = PathBuf::from(&data_dir);
            let state = handle.state::<AppState>();

            // 1. Set the data directory
            if let Ok(mut session) = state.session.write() {
                session.data_directory = Some(data_dir.clone());
            }

            // 2. Load the SNIRF file
            let snirf_path = data_dir.join("input/nirs/preprocessed.snirf");
            //load_snirf_into(snirf_path.to_str().unwrap_or(""), &state, &handle);

            // 3. Load subject anatomy
            let anatomy_path = data_dir.join("input/sub-116_ses-BL_T1w.nii.gz");
            //load_subject_anatomy_into(anatomy_path.to_str().unwrap_or(""), &state);
        });
    }

    Ok(())
}

fn load_snirf_into(path: &str, state: &tauri::State<AppState>, handle: &tauri::AppHandle) {
    println!("[setup] loading SNIRF: {path}");
    if !std::path::Path::new(path).is_file() {
        eprintln!("[setup] SNIRF file not found: {path}");
        return;
    }
    match parse_snirf(path) {
        Ok(snirf) => {
            println!("[setup] SNIRF loaded OK");
            let summary = SnirfSummary::from_snirf(&snirf);
            let optode_layout = OptodeLayout::from_snirf(&snirf);
            if let Ok(mut session) = state.session.write() {
                session.snirf = Some(snirf);
                session.optode_layout = Some(optode_layout);
            }
            let _ = handle.emit("snirf-loaded", summary);
        }
        Err(e) => eprintln!("[setup] parse_snirf failed: {e}"),
    }
}

fn load_subject_anatomy_into(path: &str, state: &tauri::State<AppState>) {
    match load_subject_anatomy(path) {
        Ok(anatomy) => {
            if let Ok(mut session) = state.session.write() {
                session.subject_anatomy = Some(anatomy);
            }
        }
        Err(e) => eprintln!("[setup] failed to load anatomy: {e}"),
    }
}
