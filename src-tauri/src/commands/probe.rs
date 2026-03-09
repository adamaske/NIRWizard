use serde::Serialize;
use tauri::{Emitter, State};

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
    let session = state.session.read().ok()?;
    let snirf = session.snirf.as_ref()?;

    let sources = snirf.nirs_entries[0]
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

    let detectors = snirf.nirs_entries[0]
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

    let n_sources = snirf.nirs_entries[0].probe.sources.len();
    let n_detectors = snirf.nirs_entries[0].probe.detectors.len();

    // SNIRF measurementList uses 1-based source/detector indices; convert to
    // 0-based for array indexing on the frontend.
    //
    // TODO : Get the channel data
    let channels = snirf
        .channels
        .channels
        .iter()
        .filter_map(|ch| {
            let src_idx = ch.source_id.checked_sub(1)?;
            let det_idx = ch.detector_id.checked_sub(1)?;
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

/// Called by ChannelSelector whenever the selection changes.
/// Stores the selection in session state and emits a `channels-selected` event.
#[tauri::command]
pub fn set_selected_channels(
    channel_ids: Vec<usize>,
    state: State<AppState>,
    app: tauri::AppHandle,
) {
    #[cfg(debug_assertions)]
    {
        let total = state
            .session
            .read()
            .ok()
            .and_then(|s| s.snirf.as_ref().map(|snirf| snirf.channels.channels.len()))
            .unwrap_or(0);
        println!(
            "[ChannelSelector] {}/{} selected: {:?}",
            channel_ids.len(),
            total,
            channel_ids
        );
    }

    if let Ok(mut session) = state.session.write() {
        session.selected_channels = channel_ids.clone();
    }

    let _ = app.emit("channels-selected", ChannelsSelectedPayload { channel_ids });
}
