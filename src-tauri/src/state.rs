use std::sync::RwLock;

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
                snirf: None,
                selected_channels: Vec::new(),
                pipeline: Pipeline::default(),
                cortex_scene: None,
                scalp_scene: None,
                optode_layout: None,
            }),
        }
    }
}

pub struct Session {
    pub snirf: Option<SNIRF>,
    pub selected_channels: Vec<usize>,
    pub pipeline: Pipeline,
    pub cortex_scene: Option<SceneObject>,
    pub scalp_scene: Option<SceneObject>,
    pub optode_layout: Option<OptodeLayout>,
}
