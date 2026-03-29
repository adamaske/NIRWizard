use crate::domain::snirf::{DataBlock, Measurement, NirsEntry, Optode};
use std::collections::BTreeMap;

// A channel is a unqiue par of source and detector
// It carries a view of the data to avoid copying
#[derive(Debug, Clone)]
pub struct ChannelView {
    id: usize, // always == vec index
    pub name: String,
    pub source_index: usize,
    pub detector_index: usize,
    pub measurement_indices: Vec<usize>, // indicies into datablock.measurements
}

impl ChannelView {
    pub fn id(&self) -> usize {
        self.id
    }
    // 0-based index into probe.sources vec.
    // Returns None if source_index is 0 (shouldn't happen in valid SNIRF).
    pub fn source_idx_0based(&self) -> Option<usize> {
        self.source_index.checked_sub(1)
    }

    // 0-based index into probe.detectors vec.
    pub fn detector_idx_0based(&self) -> Option<usize> {
        self.detector_index.checked_sub(1)
    }
}

// Grouped channel views for a one datablock
// channels[i].id() == i for all i
pub struct ChannelIndex {
    channels: Vec<ChannelView>,
}

impl ChannelIndex {
    pub fn build(block: &DataBlock) -> Self {
        let mut groups: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        // For each measurement, we make an entry of this source and detector
        // We store the index of the measurements belonging to the unqie pair
        for (i, m) in block.measurements.iter().enumerate() {
            groups
                .entry((m.source_index, m.detector_index))
                .or_default()
                .push(i);
        }

        let mut channels = groups
            .into_iter()
            .enumerate()
            .map(|(id, ((src, det), meas))| ChannelView {
                id,
                name: format!("S{}-D{}", src, det),
                source_index: src,
                detector_index: det,
                measurement_indices: meas,
            })
            .collect();

        ChannelIndex { channels }
    }

    pub fn get(&self, id: usize) -> Option<&ChannelView> {
        self.channels.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChannelView> {
        self.channels.iter()
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    pub fn as_slice(&self) -> &[ChannelView] {
        &self.channels
    }
}
