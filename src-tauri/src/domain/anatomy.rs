use crate::domain::scene::SceneObject;
use ndarray16::Array4;
use neuroformats::{write_mgh, FsMgh, FsMghData};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectAnatomy {
    pub skull: Option<SceneObject>,
    pub csf: Option<SceneObject>,
    pub grey_matter: Option<SceneObject>,
    pub white_matter: Option<SceneObject>,
}
