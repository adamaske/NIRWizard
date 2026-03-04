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
