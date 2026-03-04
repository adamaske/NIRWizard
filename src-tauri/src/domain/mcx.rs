use nalgebra as na;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Optical properties for a single tissue type.
/// MCX format: [mua, mus, g, n]
#[derive(Debug, Clone, Serialize)]
pub struct TissueOptics {
    pub label: String,
    pub mua: f64, // absorption coefficient (1/mm)
    pub mus: f64, // scattering coefficient (1/mm)
    pub g: f64,   // anisotropy factor
    pub n: f64,   // refractive index
}

/// Default optical properties at 830nm for fNIRS
/// (values from literature — Strangman, Boas, Sutherland 2002)
pub fn default_tissue_optics() -> Vec<TissueOptics> {
    vec![
        TissueOptics {
            label: "air".into(),
            mua: 0.0,
            mus: 0.0,
            g: 1.0,
            n: 1.0,
        },
        TissueOptics {
            label: "skull".into(),
            mua: 0.016,
            mus: 8.6,
            g: 0.9,
            n: 1.56,
        },
        TissueOptics {
            label: "csf".into(),
            mua: 0.004,
            mus: 0.009,
            g: 0.9,
            n: 1.33,
        },
        TissueOptics {
            label: "grey_matter".into(),
            mua: 0.02,
            mus: 9.0,
            g: 0.89,
            n: 1.37,
        },
        TissueOptics {
            label: "white_matter".into(),
            mua: 0.08,
            mus: 40.9,
            g: 0.84,
            n: 1.37,
        },
    ]
}

/// MCX source definition (one per optode that emits light)
#[derive(Debug, Clone, Serialize)]
pub struct McxSource {
    #[serde(rename = "Type")]
    pub source_type: String, // "pencil", "cone", "disk", etc.
    #[serde(rename = "Pos")]
    pub pos: [f64; 4], // [x, y, z, 1] in voxel coordinates
    #[serde(rename = "Dir")]
    pub dir: [f64; 4], // [dx, dy, dz, 0] direction vector
    #[serde(rename = "Param1")]
    pub param1: [f64; 4], // source-type specific parameters
}

/// MCX detector definition
#[derive(Debug, Clone, Serialize)]
pub struct McxDetector {
    #[serde(rename = "Pos")]
    pub pos: [f64; 3], // [x, y, z] in voxel coordinates
    #[serde(rename = "R")]
    pub radius: f64, // detector radius in voxels (typically 1-3)
}

/// Full MCX JSON input structure
#[derive(Debug, Clone, Serialize)]
pub struct McxInput {
    #[serde(rename = "Session")]
    pub session: McxSession,
    #[serde(rename = "Forward")]
    pub forward: McxForward,
    #[serde(rename = "Optode")]
    pub optode: McxOptode,
    #[serde(rename = "Domain")]
    pub domain: McxDomain,
}

#[derive(Debug, Clone, Serialize)]
pub struct McxSession {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Photons")]
    pub photons: u64,
    #[serde(rename = "DoMismatch")]
    pub do_mismatch: u8,
    #[serde(rename = "DoAutoThread")]
    pub do_auto_thread: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct McxForward {
    #[serde(rename = "T0")]
    pub t0: f64,
    #[serde(rename = "T1")]
    pub t1: f64,
    #[serde(rename = "Dt")]
    pub dt: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct McxOptode {
    #[serde(rename = "Source")]
    pub source: McxSource,
    #[serde(rename = "Detector")]
    pub detectors: Vec<McxDetector>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McxDomain {
    #[serde(rename = "VolumeFile")]
    pub volume_file: String,
    #[serde(rename = "Dim")]
    pub dim: [usize; 3],
    #[serde(rename = "OriginType")]
    pub origin_type: u8,
    #[serde(rename = "Media")]
    pub media: Vec<McxMedia>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McxMedia {
    pub mua: f64,
    pub mus: f64,
    pub g: f64,
    pub n: f64,
}
