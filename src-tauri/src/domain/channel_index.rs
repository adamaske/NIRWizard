use crate::domain::snirf::Snirf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelKey {
    // TOOD : What identifies a channel?
    // What nirs and datablock does it belong to
    pub nirs_entry: usize,
    pub block: usize,
    // What source and detector pair does it belong to
    pub source_idx: usize,
    pub detector_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignalSpec {
    HbO,
    HbR,
    Wavelength(u32),
}

// A single signal, is a channel and a spec
pub struct SignalKey {
    pub channel: ChannelKey,
    pub spec: SignalSpec,
}

pub struct ChannelIndex {}

impl Default for ChannelIndex {
    fn default() -> Self {
        ChannelIndex {}
    }
}

impl ChannelIndex {
    pub fn from_snirf(snirf: &Snirf) -> ChannelIndex {
        // For each entry
        //      For each block
        //          Turn measurementlists into channel data
        //
        //
        //  collect
        //  return

        ChannelIndex {}
    }
}
