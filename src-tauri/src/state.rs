use std::sync::RwLock;

use crate::domain::pipeline::Pipeline;
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
            }),
        }
    }
}

pub struct Session {
    pub snirf: Option<SNIRF>,
    pub selected_channels: Vec<usize>,
    pub pipeline: Pipeline,
}
