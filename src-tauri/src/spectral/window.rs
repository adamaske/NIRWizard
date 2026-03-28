use std::f64::consts::PI;

#[derive(Clone, Copy)]
pub enum WindowType {
    Hann,
    Hamming,
    Blackman,
}

pub fn ApplyWindow(data: &[f64], window: WindowType) -> Vec<f64> {
    let n = data.len() as f64;
    data.iter()
        .enumerate()
        .map(|(i, &x)| {
            let w = match window {
                WindowType::Hann => 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1.0)).cos()),
                WindowType::Hamming => 0.54 - 0.46 * (2.0 * PI * i as f64 / (n - 1.0)).cos(),
                WindowType::Blackman => {
                    0.42 - 0.5 * (2.0 * PI * i as f64 / (n - 1.0)).cos()
                        + 0.08 * (4.0 * PI * i as f64 / (n - 1.0)).cos()
                }
            };
            x * w
        })
        .collect()
}
