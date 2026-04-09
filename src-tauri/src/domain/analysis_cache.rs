use crate::domain::channel_index::ChannelIndex;
pub struct AnalysisCache {}

impl Default for AnalysisCache {
    fn default() -> Self {
        AnalysisCache {}
    }
}

impl AnalysisCache {
    pub fn from_channel_index(index: &ChannelIndex) -> AnalysisCache {
        AnalysisCache {}
    }
}
