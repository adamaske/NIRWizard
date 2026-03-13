use crate::domain::voxel::{VoxelSlicePayload, VoxelVolumeInfo};
use crate::state::AppState;

#[tauri::command]
pub fn list_voxel_volumes(state: tauri::State<AppState>) -> Vec<String> {
    let Ok(session) = state.anatomy.read() else {
        return vec![];
    };
    session.voxel_volumes.keys().cloned().collect()
}

#[tauri::command]
pub fn get_voxel_volume_info(
    name: String,
    state: tauri::State<AppState>,
) -> Result<VoxelVolumeInfo, String> {
    let session = state.anatomy.read().map_err(|e| e.to_string())?;
    session
        .voxel_volumes
        .get(&name)
        .map(|v| v.to_info())
        .ok_or_else(|| format!("Voxel volume '{name}' not found"))
}

#[tauri::command]
pub fn get_voxel_slice(
    name: String,
    axis: String,
    index: usize,
    state: tauri::State<AppState>,
) -> Result<VoxelSlicePayload, String> {
    let session = state.anatomy.read().map_err(|e| e.to_string())?;
    let vol = session
        .voxel_volumes
        .get(&name)
        .ok_or_else(|| format!("Voxel volume '{name}' not found"))?;

    let [nx, ny, nz] = vol.dims;
    let axis_char = axis.chars().next().unwrap_or('z');

    let (width, height, labels) = match axis_char {
        'x' => {
            let i = index.min(nx.saturating_sub(1));
            let mut out = Vec::with_capacity(ny * nz);
            for k in 0..nz {
                for j in 0..ny {
                    out.push(vol.labels[i + j * nx + k * nx * ny]);
                }
            }
            (ny, nz, out)
        }
        'y' => {
            let j = index.min(ny.saturating_sub(1));
            let mut out = Vec::with_capacity(nx * nz);
            for k in 0..nz {
                for i in 0..nx {
                    out.push(vol.labels[i + j * nx + k * nx * ny]);
                }
            }
            (nx, nz, out)
        }
        _ => {
            // 'z'
            let k = index.min(nz.saturating_sub(1));
            let mut out = Vec::with_capacity(nx * ny);
            for j in 0..ny {
                for i in 0..nx {
                    out.push(vol.labels[i + j * nx + k * nx * ny]);
                }
            }
            (nx, ny, out)
        }
    };

    Ok(VoxelSlicePayload {
        labels,
        width,
        height,
        axis: axis_char,
        index,
    })
}
