use crate::domain::snirf::Snirf;
use std::sync::RwLock;

pub struct SessionState {
    pub session: RwLock<Session>,
}

pub struct Session {
    pub snirf: Option<Snirf>,
    // TODO : Channel view
    // frequency view, etc
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState {
            session: RwLock::new(Session { snirf: None }),
        }
    }
}
