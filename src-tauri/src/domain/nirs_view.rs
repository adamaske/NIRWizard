use crate::domain::snirf::{DataBlock, Measurement, NirsEntry, Optode};
use std::collections::BTreeMap;

// A channel is a unqiue par of source and detector
// It carries a view of the data to avoid copying
#[derive(Debug, Clone)]
pub struct ChannelView {
    pub id: usize, // always == vec index
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

pub fn build_channel_views(block: &DataBlock) -> Vec<ChannelView> {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    // For each measurement, we make an entry of this source and detector
    // We store the index of the measurements belonging to the unqie pair
    for (i, m) in block.measurements.iter().enumerate() {
        groups
            .entry((m.source_index, m.detector_index))
            .or_default()
            .push(i);
    }

    // For each unique source-detecotr pair in groups
    groups
        .into_iter()
        .enumerate()
        .map(|(id, ((src, det), measurement_indices))| ChannelView {
            id,
            name: format!("S{}-D{}", src, det),
            source_index: src,
            detector_index: det,
            measurement_indices,
        })
        .collect()
}

// 'a borrows the nirsentry, meaning no duplication of data
pub struct NirsView<'a> {
    pub entry: &'a NirsEntry,
    pub channels: Vec<Vec<ChannelView>>,
}

impl<'a> NirsView<'a> {
    // Calls build_channel_views on each datablock
    // returns a vector of channel view vectors
    pub fn new(entry: &'a NirsEntry) -> Self {
        let channels = entry.data_blocks.iter().map(build_channel_views).collect();
        NirsView { entry, channels }
    }

    // Usually we are only interested in the 0th element
    pub fn channels_block0(&self) -> &[ChannelView] {
        self.channels.first().map(|v| v.as_slice()).unwrap_or(&[])
    }
    // Time for the first block
    pub fn time(&self) -> &[f64] {
        self.entry
            .data_blocks
            .first()
            .map(|b| b.time.as_slice())
            .unwrap_or(&[])
    }

    /// Data block 0.
    pub fn block0(&self) -> Option<&DataBlock> {
        self.entry.data_blocks.first()
    }

    pub fn channel_count(&self) -> usize {
        self.channels_block0().len()
    }

    pub fn sampling_rate(&self) -> f64 {
        let t = self.time();
        if t.len() >= 2 {
            1.0 / (t[1] - t[0])
        } else {
            0.0
        }
    }

    pub fn duration(&self) -> f64 {
        // Gets the last elemnt (s), or else 0.0
        self.time().last().copied().unwrap_or(0.0)
    }

    pub fn timepoints(&self) -> usize {
        self.time().len()
    }

    // ── Block-indexed accessors ───────────────────────────────────────────────

    pub fn block_count(&self) -> usize {
        self.entry.data_blocks.len()
    }

    pub fn block_at(&self, idx: usize) -> Option<&DataBlock> {
        self.entry.data_blocks.get(idx)
    }

    pub fn channels_at(&self, idx: usize) -> &[ChannelView] {
        self.channels.get(idx).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn time_at(&self, idx: usize) -> &[f64] {
        self.entry
            .data_blocks
            .get(idx)
            .map(|b| b.time.as_slice())
            .unwrap_or(&[])
    }

    pub fn sampling_rate_at(&self, idx: usize) -> f64 {
        let t = self.time_at(idx);
        if t.len() >= 2 {
            1.0 / (t[1] - t[0])
        } else {
            0.0
        }
    }

    pub fn duration_at(&self, idx: usize) -> f64 {
        self.time_at(idx).last().copied().unwrap_or(0.0)
    }

    pub fn data_kind_at(&self, idx: usize) -> DataKind {
        let block = match self.block_at(idx) {
            Some(b) => b,
            None => return DataKind::Empty,
        };
        if let Some(m) = block.measurements.first() {
            match m.data_type {
                99999 if m.wavelength_index.is_some() => DataKind::OpticalDensity,
                99999 => DataKind::ProcessedHemoglobin,
                _ => DataKind::RawCW,
            }
        } else {
            DataKind::Empty
        }
    }

    pub fn channel_data_at(
        &self,
        block_idx: usize,
        channel: &ChannelView,
        wavelength_position: usize,
    ) -> Option<&[f64]> {
        let block = self.block_at(block_idx)?;
        let &meas_idx = channel.measurement_indices.get(wavelength_position)?;
        Some(&block.measurements[meas_idx].data)
    }

    pub fn channel_measurement_at(
        &self,
        block_idx: usize,
        channel: &ChannelView,
        wavelength_position: usize,
    ) -> Option<&Measurement> {
        let block = self.block_at(block_idx)?;
        let &meas_idx = channel.measurement_indices.get(wavelength_position)?;
        Some(&block.measurements[meas_idx])
    }

    /// For blocks that store processed data without wavelength labels, returns the
    /// measurement position (0 or 1) within a channel that corresponds to the
    /// HbO-sensitive (longer) wavelength, by cross-referencing the first block
    /// that has wavelength indices.  Returns `None` if undeterminable.
    pub fn hbo_position_from_reference(&self) -> Option<usize> {
        let ref_block = self
            .entry
            .data_blocks
            .iter()
            .find(|b| b.measurements.iter().any(|m| m.wavelength_index.is_some()))?;

        let ref_channels = build_channel_views(ref_block);
        let ref_ch = ref_channels.first()?;

        let wl_at = |pos: usize| -> Option<f64> {
            let &meas_idx = ref_ch.measurement_indices.get(pos)?;
            let m = ref_block.measurements.get(meas_idx)?;
            m.wavelength_index.and_then(|i| self.wavelength_nm(i))
        };

        let wl0 = wl_at(0)?;
        let wl1 = wl_at(1)?;
        // HbO always has the longer wavelength.
        Some(if wl0 >= wl1 { 0 } else { 1 })
    }

    pub fn hbo_data_at(&self, block_idx: usize, channel: &ChannelView) -> Option<&[f64]> {
        let block = self.block_at(block_idx)?;
        channel.measurement_indices.iter().find_map(|&idx| {
            let m = &block.measurements[idx];
            match self.signal_kind(m) {
                SignalKind::Hemoglobin(HemoType::HbO) => Some(m.data.as_slice()),
                _ => None,
            }
        })
    }

    pub fn hbr_data_at(&self, block_idx: usize, channel: &ChannelView) -> Option<&[f64]> {
        let block = self.block_at(block_idx)?;
        channel.measurement_indices.iter().find_map(|&idx| {
            let m = &block.measurements[idx];
            match self.signal_kind(m) {
                SignalKind::Hemoglobin(HemoType::HbR) => Some(m.data.as_slice()),
                _ => None,
            }
        })
    }

    // Channel data access
    // Gets the timeseires data from a channelview based on its
    // indicies into the measurements list
    pub fn channel_data(
        &self,
        channel: &ChannelView,
        wavelength_position: usize,
    ) -> Option<&[f64]> {
        let block = self.block0()?;
        let &meas_idx = channel.measurement_indices.get(wavelength_position)?;
        Some(&block.measurements[meas_idx].data)
    }

    // Get the measurement struct for a given channel and wavelength
    pub fn channel_measurement(
        &self,
        channel: &ChannelView,
        wavelength_position: usize,
    ) -> Option<&Measurement> {
        let block = self.block0()?;
        let &meas_idx = channel.measurement_indices.get(wavelength_position)?;
        Some(&block.measurements[meas_idx])
    }

    // We want the actual nm for a wavelength
    pub fn wavelength_nm(&self, wavelength_index_1based: usize) -> Option<f64> {
        self.entry
            .probe
            .wavelengths
            .get(wavelength_index_1based.checked_sub(1)?)
            .copied()
    }

    pub fn source_optode(&self, channel: &ChannelView) -> Option<&Optode> {
        let idx = channel.source_idx_0based()?;
        self.entry.probe.sources.get(idx)
    }

    pub fn detector_optode(&self, channel: &ChannelView) -> Option<&Optode> {
        let idx = channel.detector_idx_0based()?;
        self.entry.probe.detectors.get(idx)
    }
}

// TOOD: What does partialEq do?
#[derive(Debug, Clone, PartialEq)]
pub enum SignalKind {
    //
    RawAtWavelength(f64),
    Hemoglobin(HemoType),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HemoType {
    HbO,
    HbR,
    HbT,
    Other,
}

impl NirsView<'_> {
    // Determine the signal kind for a measurement based on SNIRF dataType
    // // and dataTypeLabel fields, checks datatypelabel match, the constructs enum
    pub fn signal_kind(&self, measurement: &Measurement) -> SignalKind {
        if measurement.data_type == 99999 {
            let hemo = match measurement.data_type_label.to_lowercase().as_str() {
                "hbo" | "dod hbo" => HemoType::HbO,
                "hbr" | "dod hbr" => HemoType::HbR,
                "hbt" => HemoType::HbT,
                _ => HemoType::Other,
            };
            SignalKind::Hemoglobin(hemo)
        } else {
            let wl = measurement
                .wavelength_index
                .and_then(|i| self.wavelength_nm(i))
                .unwrap_or(0.0);
            SignalKind::RawAtWavelength(wl)
        }
    }

    pub fn hbo_data(&self, channel: &ChannelView) -> Option<&[f64]> {
        let block = self.block0()?;
        // Check both measurement indices, and find the HbO
        // if we have raw data returns none
        channel.measurement_indices.iter().find_map(|&idx| {
            let m = &block.measurements[idx];
            match self.signal_kind(m) {
                SignalKind::Hemoglobin(HemoType::HbO) => Some(m.data.as_slice()),
                _ => None,
            }
        })
    }

    pub fn hbr_data(&self, channel: &ChannelView) -> Option<&[f64]> {
        let block = self.block0()?;
        // Check both measurement indices, and find the HbO
        // if we have raw data returns none
        channel.measurement_indices.iter().find_map(|&idx| {
            let m = &block.measurements[idx];
            match self.signal_kind(m) {
                SignalKind::Hemoglobin(HemoType::HbR) => Some(m.data.as_slice()),
                _ => None,
            }
        })
    }

    // Raw CW data
    pub fn raw_wavelength_pair(&self, channel: &ChannelView) -> Option<(&[f64], &[f64])> {
        let d0 = self.channel_data(channel, 0)?;
        let d1 = self.channel_data(channel, 1)?;
        Some((d0, d1))
    }

    pub fn data_kind(&self) -> DataKind {
        self.data_kind_at(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataKind {
    /// Raw CW amplitude, data_type = 1. Two series per channel by wavelength.
    RawCW,
    /// Optical density (dOD), data_type = 99999 with wavelengthIndex present.
    /// Two series per channel by wavelength, same rendering path as RawCW.
    OpticalDensity,
    /// Processed haemoglobin concentration, data_type = 99999, no wavelengthIndex.
    /// Two series per channel: HbO and HbR (by label or position).
    ProcessedHemoglobin,
    Empty,
}
