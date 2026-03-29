use crate::domain::nirs_view::{DataKind, NirsView};
use crate::state::session::SessionState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Debug)]
pub struct ChannelPayload {
    pub id: usize,
    pub name: String,
    /// For processed data: HbO timeseries. For raw: wavelength 1 data.
    pub series_a: Vec<f64>,
    /// For processed data: HbR timeseries. For raw: wavelength 2 data.
    pub series_b: Vec<f64>,
    /// Labels for the two series, so the frontend knows what to display.
    pub series_a_label: String,
    pub series_b_label: String,
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
    pub data_kind: String,
    pub channels: Vec<ChannelPayload>,
    pub events: Vec<EventPayload>,
    pub block_index: usize,
}

#[tauri::command]
pub fn get_timeseries_data(state: State<SessionState>) -> Option<TimeseriesPayload> {
    let session = state.nirs.read().ok()?;
    let snirf = session.snirf.as_ref()?;
    let entry = snirf.nirs_entries.first()?;
    let view = NirsView::new(entry);

    let requested = state.selection.read().ok()?.active_block;
    let block_idx = requested.min(view.block_count().saturating_sub(1));

    let time = view.time_at(block_idx).to_vec();
    let data_kind = view.data_kind_at(block_idx);

    let channels: Vec<ChannelPayload> = view
        .channels_at(block_idx)
        .iter()
        .map(|ch| match data_kind {
            DataKind::ProcessedHemoglobin => {
                // Try label-based HbO/HbR lookup first (labelled files).
                // Fall back to positional for files that store processed data with
                // empty labels: derive HbO position from the wavelength ordering in
                // the reference (OD/raw) block so we assign the correct series.
                let hbo_pos = view.hbo_position_from_reference().unwrap_or(0);
                let hbr_pos = 1 - hbo_pos;
                let hbo = view
                    .hbo_data_at(block_idx, ch)
                    .or_else(|| view.channel_data_at(block_idx, ch, hbo_pos))
                    .unwrap_or(&[])
                    .to_vec();
                let hbr = view
                    .hbr_data_at(block_idx, ch)
                    .or_else(|| view.channel_data_at(block_idx, ch, hbr_pos))
                    .unwrap_or(&[])
                    .to_vec();
                ChannelPayload {
                    id: ch.id,
                    name: ch.name.clone(),
                    series_a: hbo,
                    series_b: hbr,
                    series_a_label: "HbO".into(),
                    series_b_label: "HbR".into(),
                }
            }
            // RawCW and OpticalDensity both carry two series per wavelength.
            // HbO always has the longer wavelength → series_a (red).
            // HbR always has the shorter wavelength → series_b (blue).
            DataKind::RawCW | DataKind::OpticalDensity => {
                let m0 = view.channel_measurement_at(block_idx, ch, 0);
                let m1 = view.channel_measurement_at(block_idx, ch, 1);
                let wl0 = m0
                    .and_then(|m| m.wavelength_index)
                    .and_then(|i| view.wavelength_nm(i))
                    .unwrap_or(0.0);
                let wl1 = m1
                    .and_then(|m| m.wavelength_index)
                    .and_then(|i| view.wavelength_nm(i))
                    .unwrap_or(0.0);
                let d0 = view
                    .channel_data_at(block_idx, ch, 0)
                    .unwrap_or(&[])
                    .to_vec();
                let d1 = view
                    .channel_data_at(block_idx, ch, 1)
                    .unwrap_or(&[])
                    .to_vec();

                // Longer wavelength = HbO-sensitive (series_a / red)
                let (a_data, a_wl, b_data, b_wl) = if wl0 >= wl1 {
                    (d0, wl0, d1, wl1)
                } else {
                    (d1, wl1, d0, wl0)
                };

                let prefix = if data_kind == DataKind::OpticalDensity {
                    "dOD "
                } else {
                    ""
                };
                ChannelPayload {
                    id: ch.id,
                    name: ch.name.clone(),
                    series_a: a_data,
                    series_b: b_data,
                    series_a_label: format!("{}{:.0} nm", prefix, a_wl),
                    series_b_label: format!("{}{:.0} nm", prefix, b_wl),
                }
            }
            DataKind::Empty => ChannelPayload {
                id: ch.id,
                name: ch.name.clone(),
                series_a: vec![],
                series_b: vec![],
                series_a_label: String::new(),
                series_b_label: String::new(),
            },
        })
        .collect();

    let events = entry
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

    let kind_str = match data_kind {
        DataKind::RawCW => "raw_cw",
        DataKind::OpticalDensity => "optical_density",
        DataKind::ProcessedHemoglobin => "processed_hemoglobin",
        DataKind::Empty => "empty",
    };

    Some(TimeseriesPayload {
        time,
        data_kind: kind_str.to_string(),
        channels,
        events,
        block_index: block_idx,
    })
}

#[tauri::command]
pub fn set_cursor_timepoint(time: f64, index: usize) {
    #[cfg(debug_assertions)]
    println!("[cursor] timepoint = {:.4} s  (index {})", time, index);
}
