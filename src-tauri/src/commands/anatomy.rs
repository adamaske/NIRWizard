use tauri::Emitter;
use serde::Serialize;

use crate::io::anatomy_importer::load_subject_anatomy;
use crate::state::AppState;

#[derive(Serialize, Clone)]
pub struct AnatomyLoadedPayload {
    pub layers: Vec<String>,
}

#[tauri::command]
pub fn load_mri(
    path: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let anatomy = load_subject_anatomy(&path)?;

    let layers: Vec<String> = [
        ("skull",        anatomy.skull.is_some()),
        ("csf",          anatomy.csf.is_some()),
        ("grey_matter",  anatomy.grey_matter.is_some()),
        ("white_matter", anatomy.white_matter.is_some()),
    ]
    .iter()
    .filter_map(|(name, present)| if *present { Some(name.to_string()) } else { None })
    .collect();

    let mut session = state.session.write().map_err(|e| e.to_string())?;
    session.subject_anatomy = Some(anatomy);
    drop(session);

    app.emit("anatomy-loaded", AnatomyLoadedPayload { layers })
        .map_err(|e| e.to_string())?;

    Ok(())
}
