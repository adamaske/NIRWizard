use std::collections::{BTreeSet, HashMap};

use nalgebra as na;
use serde::{Deserialize, Serialize};

/// An in-memory labelled voxel volume.
pub struct VoxelVolume {
    pub name: String,
    /// [nx, ny, nz]
    pub dims: [usize; 3],
    /// Voxel-to-RAS world transform.
    pub vox2ras: na::Matrix4<f64>,
    /// Flat label array: labels[x + y*nx + z*nx*ny]
    pub labels: Vec<u8>,
    /// Human-readable name per label value (0 = background, omit).
    pub label_names: HashMap<u8, String>,
}

/// Lightweight metadata sent to the frontend on load.
#[derive(Serialize, Clone, Debug)]
pub struct VoxelVolumeInfo {
    pub name: String,
    pub dims: [usize; 3],
    /// Column-major 4×4 vox2ras (16 f64) — pass straight to THREE.Matrix4.fromArray().
    pub vox2ras: [f64; 16],
    /// Non-zero label values that actually appear in the volume (sorted).
    pub labels_present: Vec<u8>,
    pub label_names: HashMap<u8, String>,
}

impl VoxelVolume {
    pub fn to_info(&self) -> VoxelVolumeInfo {
        let m = &self.vox2ras;
        // Column-major layout expected by Three.js Matrix4.fromArray()
        let vox2ras = [
            m[(0,0)], m[(1,0)], m[(2,0)], m[(3,0)],
            m[(0,1)], m[(1,1)], m[(2,1)], m[(3,1)],
            m[(0,2)], m[(1,2)], m[(2,2)], m[(3,2)],
            m[(0,3)], m[(1,3)], m[(2,3)], m[(3,3)],
        ];
        let labels_present: Vec<u8> = self
            .labels
            .iter()
            .copied()
            .filter(|&l| l != 0)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        VoxelVolumeInfo {
            name: self.name.clone(),
            dims: self.dims,
            vox2ras,
            labels_present,
            label_names: self.label_names.clone(),
        }
    }
}

/// A single 2-D slice of label values extracted from the volume.
#[derive(Serialize)]
pub struct VoxelSlicePayload {
    /// Flat label values, row-major [width * height].
    pub labels: Vec<u8>,
    pub width: usize,
    pub height: usize,
    /// Axis character: 'x', 'y', or 'z'.
    pub axis: char,
    pub index: usize,
}
