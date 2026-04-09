use crate::domain::channel_index::ChannelIndex;

pub struct AnalysisCache {}

impl Default for AnalysisCache {
    fn default() -> Self {
        AnalysisCache {}
    }
}

impl AnalysisCache {
    pub fn from_channel_index(_index: &ChannelIndex) -> AnalysisCache {
        // Starts empty — results are lazily populated when commands request them.
        // Pre-allocation based on index.block_count() / channel count can be added
        // when sub-caches (frequency, spectrogram) are implemented.
        AnalysisCache {}
    }
}
