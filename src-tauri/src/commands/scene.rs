use serde::Serialize;
use tauri::Emitter;

use crate::domain::mesh::MeshGeometry;
use crate::domain::probe::OptodeLayout;
use crate::domain::scene::SceneObject;
use crate::io::mesh_importer;
use crate::state::AppState;

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

fn load_obj_into(
    path: &str,
    state: &tauri::State<AppState>,
    cortex: bool,
) -> Result<SceneObjectSummary, String> {
    let mesh = mesh_importer::load_mesh(path)?;
    let name = mesh.name.clone();
    let filepath = mesh.filepath.clone();
    let vertex_count = mesh.geometry.verts.len();
    let triangle_count = mesh.geometry.indices.len() / 3;
    let obj = SceneObject::new(name.clone(), mesh);
    let mut session = state.session.write().map_err(|e| e.to_string())?;
    if cortex {
        session.cortex_scene = Some(obj);
    } else {
        session.scalp_scene = Some(obj);
    }
    Ok(SceneObjectSummary { name, vertex_count, triangle_count, filepath })
}

#[tauri::command]
pub fn load_cortex_obj(
    path: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<SceneObjectSummary, String> {
    let summary = load_obj_into(&path, &state, true)?;
    app.emit("cortex-loaded", summary.clone()).map_err(|e| e.to_string())?;
    Ok(summary)
}

#[tauri::command]
pub fn load_scalp_obj(
    path: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
) -> Result<SceneObjectSummary, String> {
    let summary = load_obj_into(&path, &state, false)?;
    app.emit("scalp-loaded", summary.clone()).map_err(|e| e.to_string())?;
    Ok(summary)
}

#[tauri::command]
pub fn get_cortex_geometry(state: tauri::State<AppState>) -> Option<MeshGeometryPayload> {
    let session = state.session.read().ok()?;
    Some(to_payload(&session.cortex_scene.as_ref()?.mesh.geometry))
}

#[tauri::command]
pub fn get_scalp_geometry(state: tauri::State<AppState>) -> Option<MeshGeometryPayload> {
    let session = state.session.read().ok()?;
    Some(to_payload(&session.scalp_scene.as_ref()?.mesh.geometry))
}

#[tauri::command]
pub fn get_optode_layout_3d(state: tauri::State<AppState>) -> Option<OptodeLayout> {
    let session = state.session.read().ok()?;
    session.optode_layout.clone()
}
