use serde::Serialize;
use tauri::Emitter;

use crate::io::anatomy_importer::{load_head_labels_volume, load_subject_anatomy};
use crate::state::state_old::AppState;

#[derive(Serialize, Clone)]
pub struct AnatomyLoadedPayload {
    pub layers: Vec<String>,
    pub voxel_volumes: Vec<String>,
}

#[tauri::command]
pub fn load_mri(
    path: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let anatomy = load_subject_anatomy(&path)?;

    let layers: Vec<String> = [
        ("skull", anatomy.skull.is_some()),
        ("csf", anatomy.csf.is_some()),
        ("grey_matter", anatomy.grey_matter.is_some()),
        ("white_matter", anatomy.white_matter.is_some()),
    ]
    .iter()
    .filter_map(|(name, present)| {
        if *present {
            Some(name.to_string())
        } else {
            None
        }
    })
    .collect();

    // Load head_labels.mgz into the voxel volume store if it exists
    let mut voxel_volumes = Vec::new();
    if let Some(ref mgz_path) = anatomy.labels_mgz_path {
        match load_head_labels_volume(mgz_path) {
            Ok(vol) => {
                voxel_volumes.push(vol.name.clone());
                let mut anatomy = state.anatomy.write().map_err(|e| e.to_string())?;
                anatomy.voxel_volumes.insert(vol.name.clone(), vol);
            }
            Err(e) => eprintln!("[anatomy] Could not load head_labels.mgz: {e}"),
        }
    }

    // ?
    //let mut anatomy = state.anatomy.write().map_err(|e| e.to_string())?;
    //session.subject_anatomy = Some(anatomy);
    //drop(session);

    app.emit(
        "anatomy-loaded",
        AnatomyLoadedPayload {
            layers,
            voxel_volumes,
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
