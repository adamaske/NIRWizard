use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SNIRF {
    pub fd: FileDescriptor,
    pub metadata: Metadata,
    pub wavelengths: Wavelengths,
    pub channels: ChannelData,
    pub probe: Probe,
    pub events: Events,
    pub biosignals: BiosignalData,
}

impl std::fmt::Display for SNIRF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SNIRF File:  {}", self.fd.name)?;
        writeln!(f, "  Path:      {}", self.fd.path)?;
        writeln!(f, "  Metadata ({}):", self.metadata.tags.len())?;
        self.metadata
            .tags
            .iter()
            .try_for_each(|t| writeln!(f, "    {:<24} {}", t.name, t.value))?;
        writeln!(
            f,
            "  Channels:  {}  ({} sources, {} detectors)",
            self.channels.channels.len(),
            self.probe.sources.len(),
            self.probe.detectors.len()
        )?;
        writeln!(f, "  Timepoints:{}", self.channels.time.len())?;
        writeln!(
            f,
            "  Wavelengths: HbO {} nm  HbR {} nm",
            self.wavelengths.hbo_wl, self.wavelengths.hbr_wl
        )?;
        writeln!(f, "  Events ({}):", self.events.events.len())?;
        self.events
            .events
            .iter()
            .try_for_each(|e| writeln!(f, "    {:<24} {} markers", e.name, e.markers.len()))?;
        write!(f, "  Aux:       {}", self.biosignals.auxilaries.len())
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
// Metadata
// =========================
#[derive(Serialize, Debug)]
pub struct MetadataTag {
    pub name: String,
    pub value: String,
}
#[derive(Serialize, Debug)]
pub struct Metadata {
    // Placeholder for actual metadata fields
    // e.g., subject_id, session_id, etc.
    pub tags: Vec<MetadataTag>,
}

// =========================
// Time Series Data
// =========================
#[derive(Serialize, Debug)]
pub struct TimeSeriesData {
    pub time: Vec<f64>,      // Time vector
    pub data: Vec<Vec<f64>>, // Channels x timepoints
}

// =========================
// Wavelengths
// =========================
#[derive(Serialize, Debug)]
pub struct Wavelengths {
    pub hbo_wl: usize,
    pub hbr_wl: usize,
}

#[derive(Serialize, Debug)]
pub struct Channel {
    // What does a channel have?
    // We have channel index i.e 0, 1, 2...
    // We have channel name i.e S1-D2, S1-D3
    // We have source_id and detector_id
    pub id: usize,
    pub name: String,
    pub source_id: usize,
    pub detector_id: usize,
    pub hbo: Vec<f64>,
    pub hbr: Vec<f64>,
}

#[derive(Serialize, Debug)]
pub struct ChannelData {
    pub time: Vec<f64>,
    pub channels: Vec<Channel>,
}

// =========================
// Probe & Optode Layout
// =========================

#[derive(Serialize, Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Debug)]
pub struct Optode {
    pub name: String,
    pub id: usize,
    pub pos_3d: Vec3,
    pub pos_2d: Vec2,
}

// EXAMPLE But we need more robust abstraction and implemetnation
#[derive(Serialize, Debug)]
pub struct Probe {
    pub sources: Vec<Optode>,   // 3D coordinates of sources
    pub detectors: Vec<Optode>, // 3D coordinates of detectors
}

// =========================
// Events / Markers / Triggers
// =========================
#[derive(Serialize, Debug)]
pub struct EventMarker {
    pub onset: f64,    // seconds
    pub duration: f64, // seconds
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct Event {
    pub name: String,
    pub markers: Vec<EventMarker>,
}

#[derive(Serialize, Debug)]
pub struct Events {
    pub events: Vec<Event>,
}

// =========================
// Biosignal / Auxiliary Data
// =========================
#[derive(Serialize, Debug)]
pub struct BiosignalData {
    pub time: Vec<f64>,                 // Time vector
    pub auxilaries: Vec<AuxiliaryData>, // List of auxiliary signals (e.g., accelerometer, heart rate)
}

#[derive(Serialize, Debug)]
pub struct AuxiliaryData {
    pub name: String,
    pub data: Vec<f64>,
}
