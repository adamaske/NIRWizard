use std::iter::empty;

use crate::{domain::probe::Channel, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct ChannelPayload {
    pub id: usize,
    pub name: String,
    pub hbo: Vec<f64>,
    pub hbr: Vec<f64>,
}

#[derive(Serialize, Debug)]
pub struct EventMarkerPayload {
    pub onset: f64,
    pub duration: f64,
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct EventPayload {
    pub name: String,
    pub markers: Vec<EventMarkerPayload>,
}

#[derive(Serialize, Debug)]
pub struct TimeseriesPayload {
    pub time: Vec<f64>,
    pub channels: Vec<ChannelPayload>,
    pub events: Vec<EventPayload>,
}

#[tauri::command]
pub fn get_timeseries_data(state: tauri::State<AppState>) -> Option<TimeseriesPayload> {
    // We have a session wich owns a SNIRF struct, we access it
    let session = state.session.read().ok()?;
    let snirf = session.snirf.as_ref()?; // Get is a reference

    // The time axis is shared across all channels in the block; clone it once.
    let time = snirf.nirs_entries[0].data_blocks[0].time.clone();

    // TODO : ChannelData View
    let channels: Vec<ChannelPayload> = empty();

    let events = snirf.nirs_entries[0]
        .events
        .iter()
        .map(|ev| EventPayload {
            name: ev.name.clone(),
            markers: ev
                .markers
                .iter()
                .map(|m| EventMarkerPayload {
                    onset: m.onset,
                    duration: m.duration,
                    value: m.value,
                })
                .collect(),
        })
        .collect();

    Some(TimeseriesPayload {
        time,
        channels,
        events,
    })
}

#[tauri::command]
pub fn set_cursor_timepoint(time: f64, index: usize) {
    #[cfg(debug_assertions)]
    println!("[cursor] timepoint = {:.4} s  (index {})", time, index);
}
