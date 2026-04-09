use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::domain::anatomy::SubjectAnatomy;
use crate::domain::probe::OptodeLayout;
use crate::domain::snirf::Snirf;
use crate::domain::voxel::VoxelVolume;

pub struct AppState {
    pub project: RwLock<ProjectState>,
    pub nirs: RwLock<NirsState>,
    pub anatomy: RwLock<AnatomyState>,
    pub selection: RwLock<SelectionState>,
    pub analysis: RwLock<AnalysisState>,
}
impl Default for AppState {
    fn default() -> Self {
        AppState {
            project: RwLock::new(ProjectState {
                data_directory: None,
            }),
            nirs: RwLock::new(NirsState {
                snirf: None,
                optode_layout: None,
            }),
            anatomy: RwLock::new(AnatomyState {
                subject_anatomy: None,
                voxel_volumes: HashMap::new(),
            }),
            selection: RwLock::new(SelectionState {
                selected_channels: Vec::new(),
                active_block: 0,
            }),
            analysis: RwLock::new(AnalysisState {}),
        }
    }
}

pub struct ProjectState {
    // Filepaths, workspace config, etc
    pub data_directory: Option<PathBuf>,
}
pub struct NirsState {
    // SNIRF data, dervied views
    pub snirf: Option<Snirf>,
    pub optode_layout: Option<OptodeLayout>,
}

pub struct AnatomyState {
    // meshes, voxels, MRI registration
    pub subject_anatomy: Option<SubjectAnatomy>,
    pub voxel_volumes: HashMap<String, VoxelVolume>,
}

pub struct SelectionState {
    // selected channels, time range, cursor, etc.
    pub selected_channels: Vec<usize>,
    pub active_block: usize,
}

pub struct AnalysisState {
    // results of processing, stats, etc.
    // GLM, connectivity, DOT output?
}
