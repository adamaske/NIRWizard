use crate::domain::nirs_view::NirsView;
use crate::dsp::{compute_fft_spectrum, compute_welch_psd, WindowType};
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SpectrumDTO {
    pub channel_id: usize,
    pub channel_name: String,
    pub label: String, // e.g. "HbO", "HbR", "760nm"
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
}

/// Returns FFT spectra for all selected channels.
/// Falls back to a synthetic test signal when no SNIRF file is loaded.
#[tauri::command]
pub fn get_spectrums(
    method: Option<String>,
    window: Option<String>,
    state: tauri::State<AppState>,
) -> Result<Vec<SpectrumDTO>, String> {
    let use_psd = method
        .as_deref()
        .unwrap_or("fft")
        .eq_ignore_ascii_case("psd");
    let window_type = match window.as_deref().unwrap_or("hann") {
        "hamming" => WindowType::Hamming,
        "blackman" => WindowType::Blackman,
        _ => WindowType::Hann,
    };
    let nirs = state.nirs.read().map_err(|e| e.to_string())?;
    let selection = state.selection.read().map_err(|e| e.to_string())?;

    let Some(snirf) = nirs.snirf.as_ref() else {
        // No file loaded — return the synthetic test signal as a single entry
        drop(nirs);
        drop(selection);
        let dto = make_test_spectrum();
        return Ok(vec![dto]);
    };

    let entry = snirf
        .nirs_entries
        .first()
        .ok_or("No NIRS entries in file")?;
    let view = NirsView::new(entry);
    let block_idx = selection
        .active_block
        .min(view.block_count().saturating_sub(1));
    let sample_rate = view.sampling_rate_at(block_idx);
    let channels = view.channels_at(block_idx).to_vec();
    let channels = channels.as_slice();

    // If nothing selected, use all channels; otherwise filter to selected ids
    let selected: Vec<usize> = if selection.selected_channels.is_empty() {
        (0..channels.len()).collect()
    } else {
        selection.selected_channels.clone()
    };

    let mut out = Vec::new();
    for &ch_id in &selected {
        let Some(ch) = channels.get(ch_id) else {
            continue;
        };
        // Emit one SpectrumDTO per measurement (wavelength or hemo type) in the channel
        for (pos, &meas_idx) in ch.measurement_indices.iter().enumerate() {
            let block = match view.block_at(block_idx) {
                Some(b) => b,
                None => continue,
            };
            let signal = &block.measurements[meas_idx].data;
            let label = {
                let m = &block.measurements[meas_idx];
                if m.data_type_label.is_empty() {
                    view.entry
                        .probe
                        .wavelengths
                        .get(pos)
                        .map(|wl| format!("{wl:.0}nm"))
                        .unwrap_or_else(|| format!("series{pos}"))
                } else {
                    m.data_type_label.clone()
                }
            };
            let result = if use_psd {
                let seg = 256.min(signal.len()).max(4);
                if signal.len() >= seg {
                    compute_welch_psd(signal, sample_rate, seg, seg / 2, window_type)
                } else {
                    compute_fft_spectrum(signal, sample_rate, window_type)
                }
            } else {
                compute_fft_spectrum(signal, sample_rate, window_type)
            };
            out.push(SpectrumDTO {
                channel_id: ch.id,
                channel_name: ch.name.clone(),
                label,
                frequencies: result.frequencies,
                magnitudes: result.magnitudes,
            });
        }
    }
    Ok(out)
}

fn make_test_spectrum() -> SpectrumDTO {
    let sample_rate: f64 = 10.0;
    let n = 64;
    let signal: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * i as f64 / sample_rate).sin())
        .collect();
    let result = compute_fft_spectrum(&signal, sample_rate, WindowType::Hann);
    SpectrumDTO {
        channel_id: 0,
        channel_name: "test".to_string(),
        label: "1 Hz sine".to_string(),
        frequencies: result.frequencies,
        magnitudes: result.magnitudes,
    }
}
