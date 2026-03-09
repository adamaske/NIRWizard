use crate::domain::snirf::{DataBlock, Measurement, NirsEntry, Optode};

// A channel is a unqiue par of source and detector
// It carries a view of the data to avoid copying
//
#[derive(Debug, Clone)]
pub struct ChannelView {
    pub id: usize,

    pub name: String,
    pub source_index: usize,
    pub detector_index: usize,

    // Keys into measurement lits, if we have more than 2 wavelength
    // we have a  flexible array
    pub measurement_indices: Vec<usize>,
}

impl ChannelView {
    /// 0-based index into probe.sources vec.
    /// Returns None if source_index is 0 (shouldn't happen in valid SNIRF).
    pub fn source_idx_0based(&self) -> Option<usize> {
        self.source_index.checked_sub(1)
    }

    /// 0-based index into probe.detectors vec.
    pub fn detector_idx_0based(&self) -> Option<usize> {
        self.detector_index.checked_sub(1)
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
        if measurement.data_type == 9999 {
            let hemo = match measurement.data_type_label.to_lowercase().as_str() {
                "hbo" | "dod hbo" => HemoType::HbO,
                "hbr" | "dod hbr" => HemoType::HbR,
                "hbt" => HemoType::HbT,
                _ => HemoType::Other,
            };
            SignalKind::Hemoglobin(hemo)
        } else {
            let wl = self
                .wavelength_nm(measurement.wavelength_index)
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
        let block = match self.block0() {
            Some(b) => b,
            None => return DataKind::Empty,
        };

        if let Some(m) = block.measurements.first() {
            if m.data_type == 9999 {
                DataKind::ProcessedHemoglobin
            } else {
                DataKind::RawCW
            }
        } else {
            DataKind::Empty
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataKind {
    RawCW,
    ProcessedHemoglobin,
    Empty,
}
