use std::sync::RwLock;

use crate::domain::snirf::SNIRF;

pub struct AppState {
    pub session: RwLock<Session>,
}

pub struct Session {
    pub snirf: SNIRF,
}