use crate::analysis::window::{apply_window, WindowType};
use realfft::RealFftPlanner;

#[derive(serde::Serialize)]
pub struct SpectrumResult {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
}

pub fn compute_fft_spectrum(
    signal: &[f64],
    sample_rate: f64,
    window: WindowType,
) -> SpectrumResult {
    let n = signal.len();
    let windowed = apply_window(signal, window);

    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);

    let mut input = windowed;
    let mut output = fft.make_output_vec();

    fft.process(&mut input, &mut output).unwrap();

    let magnitudes: Vec<f64> = output
        .iter()
        .map(|c| (c.re * c.re + c.im * c.im).sqrt() * 2.0 / n as f64)
        .collect();
    let freq_resolution = sample_rate / n as f64;

    let frequencies: Vec<f64> = (0..magnitudes.len())
        .map(|i| i as f64 * freq_resolution)
        .collect();

    SpectrumResult {
        frequencies,
        magnitudes,
    }
}

pub fn compute_welch_psd(
    signal: &[f64],
    sample_rate: f64,
    segment_len: usize,
    overlap: usize, // typically segment_len / 2
    window: WindowType,
) -> SpectrumResult {
    let hop = segment_len - overlap;
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(segment_len);

    let n_segments = (signal.len() - overlap) / hop;
    let n_bins = segment_len / 2 + 1;
    let mut psd = vec![0.0f64; n_bins];

    for i in 0..n_segments {
        let start = i * hop;
        let segment = &signal[start..start + segment_len];
        let windowed = apply_window(segment, window);

        let mut input = windowed;
        let mut output = fft.make_output_vec();
        fft.process(&mut input, &mut output).unwrap();

        for (j, c) in output.iter().enumerate() {
            psd[j] += (c.re * c.re + c.im * c.im) / (segment_len as f64 * sample_rate);
        }
    }

    // Average across segments
    psd.iter_mut().for_each(|v| *v /= n_segments as f64);

    let freq_resolution = sample_rate / segment_len as f64;
    let frequencies: Vec<f64> = (0..n_bins).map(|i| i as f64 * freq_resolution).collect();

    SpectrumResult {
        frequencies,
        magnitudes: psd,
    }
}
