use crate::domain::{analysis_cache::AnalysisCache, channel_index::ChannelIndex, snirf::Snirf};
use std::sync::RwLock;

pub struct SessionState {
    pub session: RwLock<Session>,
}

pub struct Session {
    pub snirf: Option<Snirf>,
    pub channel_index: Option<ChannelIndex>,
    pub analysis_cache: AnalysisCache,
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState {
            session: RwLock::new(Session {
                snirf: None,
                channel_index: None,
                analysis_cache: AnalysisCache::default(),
            }),
        }
    }
}

impl SessionState {
    pub fn load(&self, snirf: Snirf, index: ChannelIndex, cache: AnalysisCache) {
        let mut session = self.session.write().unwrap();
        session.snirf = Some(snirf);
        session.channel_index = Some(index);
        session.analysis_cache = cache;
    }
}
