use crate::state::AppState;
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

    // The payload-time and snirf-time is equal so we can copy it.
    let time = snirf.channels.time.clone();
    // Iterate over each channel,
    // create a ChannelPayload,
    // and colelct into channels vec
    let channels = snirf
        .channels
        .channels
        .iter()
        .map(|ch| ChannelPayload {
            id: ch.id,
            name: ch.name.clone(),
            hbo: ch.hbo.clone(),
            hbr: ch.hbr.clone(),
        })
        .collect();

    let events = snirf
        .events
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

    Some(TimeseriesPayload { time, channels, events })
}

#[tauri::command]
pub fn set_cursor_timepoint(time: f64, index: usize) {
    #[cfg(debug_assertions)]
    println!("[cursor] timepoint = {:.4} s  (index {})", time, index);
}
