use crate::state::AppState;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ChannelPayload {
    pub id: usize,
    pub name: String,
    pub hbo: Vec<f64>,
    pub hbr: Vec<f64>,
}

#[derive(Serialize, Debug)]
pub struct TimeseriesPayload {
    pub time: Vec<f64>,
    pub channels: Vec<ChannelPayload>,
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

    Some(TimeseriesPayload { time, channels })
}
