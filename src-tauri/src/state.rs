use std::sync::RwLock;

use crate::domain::snirf::SNIRF;

pub struct AppState {
    pub session: RwLock<Session>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            session: RwLock::new(Session { snirf: None }),
        }
    }
}

pub struct Session {
    pub snirf: Option<SNIRF>,
}