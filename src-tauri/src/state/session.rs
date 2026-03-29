use crate::domain::channel::ChannelIndex;
use crate::domain::snirf::Snirf;
use std::sync::RwLock;

pub struct SessionState {
    inner: RwLock<SessionInner>,
}

struct SessionInner {
    snirf: Option<Snirf>,
    channel_indices: Vec<ChannelIndex>,
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState {
            inner: RwLock::new(SessionInner {
                snirf: None,
                channel_indices: Vec::new(),
            }),
        }
    }
}

impl SessionState {
    pub fn load(&self, snirf: Snirf, indices: Vec<ChannelIndex>) {
        let mut inner = self.inner.write().unwrap();
        inner.snirf = Some(snirf);
        inner.channel_indices = indices;
    }
}
