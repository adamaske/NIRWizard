use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::domain::anatomy::SubjectAnatomy;
use crate::domain::probe::OptodeLayout;
use crate::domain::snirf::SNIRF;
use crate::domain::voxel::VoxelVolume;

pub struct AppState {
    pub session: RwLock<Session>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            session: RwLock::new(Session {
                data_directory: None,
                subject_anatomy: None,
                voxel_volumes: HashMap::new(),
                snirf: None,
                optode_layout: None,
                selected_channels: Vec::new(),
            }),
        }
    }
}

pub struct Session {
    pub data_directory: Option<PathBuf>,
    pub subject_anatomy: Option<SubjectAnatomy>,
    pub voxel_volumes: HashMap<String, VoxelVolume>,
    pub snirf: Option<SNIRF>,
    pub optode_layout: Option<OptodeLayout>,
    pub selected_channels: Vec<usize>,
}
