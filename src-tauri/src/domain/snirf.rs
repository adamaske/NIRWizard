use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SNIRF{
    pub fd: FileDescriptor,
    pub timeseries: TimeSeriesData,
    pub probe: Probe,
    pub biosignals: BiosignalData,
}

impl std::fmt::Display for SNIRF{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SNIRF File:  {}", self.fd.name)?;
        writeln!(f, "  Path:      {}", self.fd.path)?;
        writeln!(f, "  Channels:  {}", self.timeseries.data.len())?;
        writeln!(f, "  Timepoints:{}", self.timeseries.time.len())?;
        writeln!(f, "  Sources:   {}", self.probe.sources.len())?;
        writeln!(f, "  Detectors: {}", self.probe.detectors.len())?;
        write!(f,   "  Aux:       {}", self.biosignals.auxilaries.len())
    }
}

// =========================
// File Descriptor
// =========================
#[derive(Serialize, Debug)]
pub struct FileDescriptor {
    pub path: String,
    pub name: String,
}

// =========================
// Time Series Data
// =========================
#[derive(Serialize, Debug)]
pub struct TimeSeriesData {
    pub time: Vec<f64>, // Time vector
    pub data: Vec<Vec<f64>>, // Channels x timepoints
}

// =========================
// Probe & Optode Layout
// =========================
#[derive(Serialize, Debug)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Debug)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// EXAMPLE But we need more robust abstraction and implemetnation
#[derive(Serialize, Debug)]
pub struct Probe {
    pub sources: Vec<Vector3D>, // 3D coordinates of sources
    pub detectors: Vec<Vector3D>, // 3D coordinates of detectors
}


// =========================
// Biosignal / Auxiliary Data
// =========================
#[derive(Serialize, Debug)]
pub struct BiosignalData {
    pub time: Vec<f64>, // Time vector
    pub auxilaries: Vec<AuxiliaryData>, // List of auxiliary signals (e.g., accelerometer, heart rate)
}

#[derive(Serialize, Debug)]
pub struct AuxiliaryData{
    pub name: String,
    pub data: Vec<f64>,
}


