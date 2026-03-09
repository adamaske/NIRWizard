use crate::domain::scene::SceneObject;
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectAnatomy {
    pub skull: Option<SceneObject>,
    pub csf: Option<SceneObject>,
    pub grey_matter: Option<SceneObject>,
    pub white_matter: Option<SceneObject>,
    /// Path to head_labels.mgz — used to load a VoxelVolume on demand.
    #[serde(skip)]
    pub labels_mgz_path: Option<std::path::PathBuf>,
}

// Voxelizaed Head Anatomy: 5 tissues
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoxelAnatomy {
    pub skull: Option<VoxelVolume>,
    pub csf: Option<VoxelVolume>,
    pub grey_matter: Option<VoxelVolume>,
    pub white_matter: Option<VoxelVolume>,
    pub scalp: Option<VoxelVolume>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoxelVolume {
    pub dimensions: (usize, usize, usize),
    pub voxel_size_mm: (f32, f32, f32),
    pub data: Vec<u8>,
    pub xd: String,
}
