use nalgebra as na;
use serde::Serialize;

use crate::domain::anatomy::SubjectAnatomy;
use crate::domain::mesh::MeshGeometry;
use crate::domain::probe::OptodeLayout;
use crate::domain::scene::{SceneObject, Transform};
use crate::state::state_old::AppState;

fn transform_from_arrays(position: [f64; 3], rotation: [f64; 3], scale: [f64; 3]) -> Transform {
    Transform {
        position: na::Vector3::new(position[0], position[1], position[2]),
        rotation: na::Vector3::new(rotation[0], rotation[1], rotation[2]),
        scale: na::Vector3::new(scale[0], scale[1], scale[2]),
    }
}

/// Lightweight summary returned to the frontend after loading a scene object.
#[derive(Serialize, Clone, Debug)]
pub struct SceneObjectSummary {
    pub name: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub filepath: String,
}

/// Flat geometry payload for Three.js BufferGeometry construction.
#[derive(Serialize)]
pub struct MeshGeometryPayload {
    pub positions: Vec<f32>, // flat [x,y,z, x,y,z, ...]
    pub normals: Vec<f32>,   // flat [nx,ny,nz, ...]
    pub indices: Vec<u32>,
}

fn to_payload(geometry: &MeshGeometry) -> MeshGeometryPayload {
    let mut positions = Vec::with_capacity(geometry.verts.len() * 3);
    let mut normals = Vec::with_capacity(geometry.verts.len() * 3);
    for v in &geometry.verts {
        positions.push(v.position.x as f32);
        positions.push(v.position.y as f32);
        positions.push(v.position.z as f32);
        normals.push(v.normal.x as f32);
        normals.push(v.normal.y as f32);
        normals.push(v.normal.z as f32);
    }
    MeshGeometryPayload {
        positions,
        normals,
        indices: geometry.indices.clone(),
    }
}

fn get_layer<'a>(anatomy: &'a SubjectAnatomy, layer: &str) -> Result<&'a SceneObject, String> {
    match layer {
        "skull" => anatomy.skull.as_ref(),
        "csf" => anatomy.csf.as_ref(),
        "grey_matter" => anatomy.grey_matter.as_ref(),
        "white_matter" => anatomy.white_matter.as_ref(),
        _ => None,
    }
    .ok_or_else(|| format!("Layer '{layer}' not loaded"))
}

fn get_layer_mut<'a>(
    anatomy: &'a mut SubjectAnatomy,
    layer: &str,
) -> Result<&'a mut SceneObject, String> {
    match layer {
        "skull" => anatomy.skull.as_mut(),
        "csf" => anatomy.csf.as_mut(),
        "grey_matter" => anatomy.grey_matter.as_mut(),
        "white_matter" => anatomy.white_matter.as_mut(),
        _ => None,
    }
    .ok_or_else(|| format!("Layer '{layer}' not loaded"))
}

#[tauri::command]
pub fn get_anatomy_geometry(
    layer: String,
    state: tauri::State<AppState>,
) -> Result<MeshGeometryPayload, String> {
    let anatomy = state.anatomy.read().map_err(|e| e.to_string())?;
    let anatomy = anatomy
        .subject_anatomy
        .as_ref()
        .ok_or("No anatomy loaded")?;
    let obj = get_layer(anatomy, &layer)?;
    Ok(to_payload(&obj.mesh.geometry))
}

#[tauri::command]
pub fn set_anatomy_transform(
    layer: String,
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut anatomy = state.anatomy.write().map_err(|e| e.to_string())?;
    let anatomy = anatomy
        .subject_anatomy
        .as_mut()
        .ok_or("No anatomy loaded")?;
    let obj = get_layer_mut(anatomy, &layer)?;
    obj.transform = transform_from_arrays(position, rotation, scale);
    Ok(())
}

#[tauri::command]
pub fn set_anatomy_opacity(
    layer: String,
    opacity: f64,
    visible: bool,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut anatomy = state.anatomy.write().map_err(|e| e.to_string())?;
    let anatomy = anatomy
        .subject_anatomy
        .as_mut()
        .ok_or("No anatomy loaded")?;
    let obj = get_layer_mut(anatomy, &layer)?;
    obj.opacity = opacity;
    obj.visible = visible;
    Ok(())
}

#[tauri::command]
pub fn get_optode_layout_3d(state: tauri::State<AppState>) -> Option<OptodeLayout> {
    let nirs = state.nirs.read().ok()?;
    nirs.optode_layout.clone()
}

#[tauri::command]
pub fn set_optode_layout_transform(
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut session = state.nirs.write().map_err(|e| e.to_string())?;
    let layout = session.optode_layout.as_mut().ok_or("No probe loaded")?;
    layout.transform = transform_from_arrays(position, rotation, scale);
    Ok(())
}

#[tauri::command]
pub fn set_optode_layout_settings(
    spread_factor: f64,
    optode_radius: f64,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut session = state.nirs.write().map_err(|e| e.to_string())?;
    let layout = session.optode_layout.as_mut().ok_or("No probe loaded")?;
    layout.settings.spread_factor = spread_factor;
    layout.settings.optode_radius = optode_radius;
    Ok(())
}
