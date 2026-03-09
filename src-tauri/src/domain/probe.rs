use crate::domain::nirs_view::NirsView;
use crate::domain::scene::Transform;
use crate::domain::snirf::SNIRF;
use nalgebra::Vector3;
use serde::Serialize;
use tauri::ipc::Channel;

/// A single optode (source or detector) with its 3D position.
#[derive(Debug, Clone, Serialize)]
pub struct Optode3D {
    pub id: usize,
    pub name: String,
    pub position: Vector3<f64>, // raw position from SNIRF (mm-scale)
}

/// A channel connecting a source to a detector by index into the sources/detectors vecs.
#[derive(Debug, Clone, Serialize)]
pub struct ChannelConnection {
    pub id: usize,
    pub source_idx: usize,   // 0-based into sources
    pub detector_idx: usize, // 0-based into detectors
}

/// Display/layout parameters the frontend uses to position and scale the probe.
#[derive(Debug, Clone, Serialize)]
pub struct ProbeDisplaySettings {
    pub spread_factor: f64,
    pub optode_radius: f64,
    pub projection_target: Vector3<f64>,
}

impl Default for ProbeDisplaySettings {
    fn default() -> Self {
        Self {
            spread_factor: 1.0,
            optode_radius: 0.005,
            projection_target: Vector3::zeros(),
        }
    }
}

/// The full 3D probe layout, constructed from a loaded SNIRF file.
/// Pure data — no render state. Three.js constructs geometry from this at load time.
#[derive(Debug, Clone, Serialize)]
pub struct OptodeLayout {
    pub sources: Vec<Optode3D>,
    pub detectors: Vec<Optode3D>,
    pub channels: Vec<ChannelConnection>,
    pub settings: ProbeDisplaySettings,
    pub transform: Transform,
}

impl OptodeLayout {
    pub fn from_snirf(snirf: &SNIRF) -> Self {
        let entry = &snirf.nirs_entries[0];
        let view = NirsView::new(entry);

        let sources: Vec<Optode3D> = entry
            .probe
            .sources
            .iter()
            .map(|o| Optode3D {
                id: o.id,
                name: o.name.clone(),
                position: o.pos_3d,
            })
            .collect();

        let detectors: Vec<Optode3D> = entry
            .probe
            .detectors
            .iter()
            .map(|o| Optode3D {
                id: o.id,
                name: o.name.clone(),
                position: o.pos_3d,
            })
            .collect();

        let n_sources = sources.len();
        let n_detectors = detectors.len();

        let channels: Vec<ChannelConnection> = view
            .channels_block0()
            .iter()
            .filter_map(|ch| {
                let src_idx = ch.source_idx_0based()?;
                let det_idx = ch.detector_idx_0based()?;

                if src_idx >= n_sources || det_idx >= n_detectors {
                    return None;
                }
                Some(ChannelConnection {
                    id: ch.id,
                    source_idx: src_idx,
                    detector_idx: det_idx,
                })
            })
            .collect();

        println!(
            "[OptodeLayout] built: {} sources, {} detectors, {} channels",
            sources.len(),
            detectors.len(),
            channels.len()
        );

        Self {
            sources,
            detectors,
            channels,
            settings: ProbeDisplaySettings::default(),
            transform: Transform::default(),
        }
    }
}
