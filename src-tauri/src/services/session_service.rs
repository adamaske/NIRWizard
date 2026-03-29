use crate::domain::channel::ChannelIndex;
use crate::domain::error::NWError;
use crate::domain::snirf::Snirf;
use crate::domain::summary::SnirfSummary;
use crate::io::snirf_parser;
use log::info;

pub struct LoadResult {
    pub snirf: Snirf,
    pub channel_indices: Vec<ChannelIndex>,
    pub summary: SnirfSummary,
}

pub fn load_snirf(path: &str) -> Result<LoadResult, NWError> {
    let snirf = snirf_parser::parse_snirf(path)?;

    let channel_indices = snirf
        .nirs_entries
        .first()
        .map(|e| e.data_blocks.iter().map(ChannelIndex::build).collect())
        .unwrap_or_default();

    let summary = SnirfSummary::from_snirf(&snirf);

    info!(
        "Loaded '{}': {} channels, {:.1}s, {} blocks",
        summary.filename,
        summary.channels,
        summary
            .data_blocks
            .first()
            .map(|b| b.duration)
            .unwrap_or(0.0),
        summary.data_blocks.len()
    );

    Ok(LoadResult {
        snirf,
        channel_indices,
        summary,
    })
}
