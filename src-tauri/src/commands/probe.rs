use serde::Serialize;
use tauri::{Emitter, State};

use crate::domain::nirs_view::NirsView;
use crate::state::AppState;

// =============================================================================
// Response types
// =============================================================================
#[derive(Serialize)]
pub struct OptodePosition {
    pub id: usize,
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize)]
pub struct ChannelTopology {
    pub id: usize,
    pub name: String,
    pub source_idx: usize,   // 0-based index into sources array
    pub detector_idx: usize, // 0-based index into detectors array
}

#[derive(Serialize)]
pub struct ProbeLayout {
    pub sources: Vec<OptodePosition>,
    pub detectors: Vec<OptodePosition>,
    pub channels: Vec<ChannelTopology>,
}

// =============================================================================
// Commands
// =============================================================================

/// Returns the 2-D probe layout (optode positions + channel topology) needed
/// by the ChannelSelector frontend component.
#[tauri::command]
pub fn get_probe_layout(state: State<AppState>) -> Option<ProbeLayout> {
    let nirs = state.nirs.read().ok()?;
    let snirf = nirs.snirf.as_ref()?;
    let entry = snirf.nirs_entries.first()?;
    let view = NirsView::new(entry);

    let n_sources = entry.probe.sources.len();
    let n_detectors = entry.probe.detectors.len();

    let sources = entry
        .probe
        .sources
        .iter()
        .map(|o| OptodePosition {
            id: o.id,
            name: o.name.clone(),
            x: o.pos_2d.x,
            y: o.pos_2d.y,
        })
        .collect();

    let detectors = entry
        .probe
        .detectors
        .iter()
        .map(|o| OptodePosition {
            id: o.id,
            name: o.name.clone(),
            x: o.pos_2d.x,
            y: o.pos_2d.y,
        })
        .collect();

    let channels = view
        .channels_block0()
        .iter()
        .filter_map(|ch| {
            let src_idx = ch.source_idx_0based()?;
            let det_idx = ch.detector_idx_0based()?;
            if src_idx >= n_sources || det_idx >= n_detectors {
                return None;
            }
            Some(ChannelTopology {
                id: ch.id,
                name: ch.name.clone(),
                source_idx: src_idx,
                detector_idx: det_idx,
            })
        })
        .collect();

    Some(ProbeLayout {
        sources,
        detectors,
        channels,
    })
}

#[derive(Serialize, Clone)]
pub struct ChannelsSelectedPayload {
    pub channel_ids: Vec<usize>,
}

#[tauri::command]
pub fn set_selected_channels(
    channel_ids: Vec<usize>,
    state: State<AppState>,
    app: tauri::AppHandle,
) {
    #[cfg(debug_assertions)]
    {
        let count = state
            .nirs
            .read()
            .ok()
            .and_then(|s| {
                let snirf = s.snirf.as_ref()?;
                let entry = snirf.nirs_entries.first()?;
                let view = NirsView::new(entry);
                Some(view.channel_count())
            })
            .unwrap_or(0);
        println!(
            "[ChannelSelector] {}/{} selected: {:?}",
            channel_ids.len(),
            count,
            channel_ids
        );
    }

    if let Ok(mut selection) = state.selection.write() {
        selection.selected_channels = channel_ids.clone();
    }

    let _ = app.emit("channels-selected", ChannelsSelectedPayload { channel_ids });
}

#[tauri::command]
pub fn set_active_block(
    index: usize,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut selection = state.selection.write().map_err(|e| e.to_string())?;
    selection.active_block = index;
    drop(selection);
    app.emit("block-changed", index).map_err(|e| e.to_string())
}
