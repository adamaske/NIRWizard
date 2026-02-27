use serde::Serialize;
use crate::domain::probe::Probe;

#[derive(Serialize, Debug)]
pub struct SNIRF{
    pub fd: FileDescriptor,
    pub data: TimeSeriesData,

    pub biosignal: BiosignalData,

    pub probe: Probe,

}

impl std::fmt::Display for SNIRF{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SNIRF File: {}, Channels: {}, Timepoints: {}", self.fd.name, self.data.data.len(), self.data.time.len())
    }
}

#[derive(Serialize, Debug)]
pub struct FileDescriptor {
    pub path: String,
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct TimeSeriesData {
    pub time: Vec<f64>, // Time vector
    pub data: Vec<Vec<f64>>, // Channels x timepoints
}

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
