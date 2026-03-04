use std::sync::RwLock;

use std::path::{Path, PathBuf};

use crate::domain::anatomy::SubjectAnatomy;
use crate::domain::pipeline::Pipeline;
use crate::domain::probe::OptodeLayout;
use crate::domain::scene::SceneObject;
use crate::domain::snirf::SNIRF;

pub struct AppState {
    pub session: RwLock<Session>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            session: RwLock::new(Session {
                data_directory: None,
                subject_anatomy: None,
                snirf: None,
                optode_layout: None,
                selected_channels: Vec::new(),
                pipeline: Pipeline::default(),
            }),
        }
    }
}

pub struct Session {
    pub data_directory: Option<PathBuf>, // This is where we can find our files.

    pub subject_anatomy: Option<SubjectAnatomy>,

    pub snirf: Option<SNIRF>,
    pub optode_layout: Option<OptodeLayout>,

    pub selected_channels: Vec<usize>,

    pub pipeline: Pipeline,
}
