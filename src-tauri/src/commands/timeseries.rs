use crate::domain::nirs_view::{DataKind, NirsView};
use crate::state::AppState;
use serde::Serialize;

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
}

#[tauri::command]
pub fn get_timeseries_data(state: tauri::State<AppState>) -> Option<TimeseriesPayload> {
    let session = state.nirs.read().ok()?;
    let snirf = session.snirf.as_ref()?;
    let entry = snirf.nirs_entries.first()?;
    let view = NirsView::new(entry);

    let time = view.time().to_vec();
    let data_kind = view.data_kind();

    let channels: Vec<ChannelPayload> = view
        .channels_block0()
        .iter()
        .map(|ch| match data_kind {
            DataKind::ProcessedHemoglobin => {
                let hbo = view.hbo_data(ch).unwrap_or(&[]).to_vec();
                let hbr = view.hbr_data(ch).unwrap_or(&[]).to_vec();
                ChannelPayload {
                    id: ch.id,
                    name: ch.name.clone(),
                    series_a: hbo,
                    series_b: hbr,
                    series_a_label: "HbO".into(),
                    series_b_label: "HbR".into(),
                }
            }
            DataKind::RawCW => {
                // Look up each measurement's actual wavelength so we can assign
                // the longer wavelength → series_a (HbO, red) and the shorter
                // wavelength → series_b (HbR, blue), regardless of file order.
                let m0 = view.channel_measurement(ch, 0);
                let m1 = view.channel_measurement(ch, 1);
                let wl0 = m0
                    .and_then(|m| view.wavelength_nm(m.wavelength_index))
                    .unwrap_or(0.0);
                let wl1 = m1
                    .and_then(|m| view.wavelength_nm(m.wavelength_index))
                    .unwrap_or(0.0);
                let d0 = view.channel_data(ch, 0).unwrap_or(&[]).to_vec();
                let d1 = view.channel_data(ch, 1).unwrap_or(&[]).to_vec();

                // Higher wavelength = HbO-sensitive (series_a / red)
                let (hbo_data, hbo_wl, hbr_data, hbr_wl) = if wl0 >= wl1 {
                    (d0, wl0, d1, wl1)
                } else {
                    (d1, wl1, d0, wl0)
                };

                ChannelPayload {
                    id: ch.id,
                    name: ch.name.clone(),
                    series_a: hbo_data,
                    series_b: hbr_data,
                    series_a_label: format!("{:.0} nm", hbo_wl),
                    series_b_label: format!("{:.0} nm", hbr_wl),
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
        DataKind::ProcessedHemoglobin => "processed_hemoglobin",
        DataKind::Empty => "empty",
    };

    Some(TimeseriesPayload {
        time,
        data_kind: kind_str.to_string(),
        channels,
        events: events,
    })
}

#[tauri::command]
pub fn set_cursor_timepoint(time: f64, index: usize) {
    #[cfg(debug_assertions)]
    println!("[cursor] timepoint = {:.4} s  (index {})", time, index);
}
