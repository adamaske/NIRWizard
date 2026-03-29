use crate::domain::nirs_view::{ChannelIndex, DataKind};
use crate::domain::snirf::Snirf;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct BlockSummary {
    pub index: usize,
    pub data_kind: String,
    pub channels: usize,
    pub timepoints: usize,
    pub sampling_rate: f64,
    pub duration: f64,
}

#[derive(Serialize, Clone)]
pub struct SnirfSummary {
    pub filename: String,
    pub format_version: String,
    pub channels: usize,
    pub sources: usize,
    pub detectors: usize,
    pub wavelengths: Vec<f64>,
    pub data_blocks: Vec<BlockSummary>,
    pub event_count: usize,
    pub aux_count: usize,
}

impl SnirfSummary {
    pub fn from_snirf(snirf: &Snirf) -> Self {
        let entry = &snirf.nirs_entries[0];

        let blocks: Vec<BlockSummary> = entry
            .data_blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                let ci = ChannelIndex::build(block);
                let sr = if block.time.len() >= 2 {
                    1.0 / (block.time[1] - block.time[0])
                } else {
                    0.0
                };
                BlockSummary {
                    index: i,
                    data_kind: DataKind::detect(block).as_str().to_string(),
                    channels: ci.len(),
                    timepoints: block.time.len(),
                    sampling_rate: sr,
                    duration: block.time.last().copied().unwrap_or(0.0),
                }
            })
            .collect();

        let first = blocks.first();
        SnirfSummary {
            filename: snirf.file_descriptor.clone(),
            format_version: snirf.format_version.clone(),
            channels: first.map(|b| b.channels).unwrap_or(0),
            sources: entry.probe.sources.len(),
            detectors: entry.probe.detectors.len(),
            wavelengths: entry.probe.wavelengths.clone(),
            data_blocks: blocks,
            event_count: entry.events.len(),
            aux_count: entry.auxiliaries.len(),
        }
    }
}
