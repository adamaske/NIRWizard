// TODO : Implement the probe information here
// We need 2D and 3D vectors to store probe locations
// We need some sort of channel = Source & Detector abstraction
use crate::domain::vector::{Vector2D, Vector3D};
use serde::Serialize;

// We need some easy way to access the correct channel based on the source and detector indices
// And vice versa

// EXAMPLE But we need more robust abstraction and implemetnation
#[derive(Serialize, Debug)]
pub struct Probe {
    pub sources: Vec<Vector3D>, // 3D coordinates of sources
    pub detectors: Vec<Vector3D>, // 3D coordinates of detectors
    pub channels: Vec<Channel>, // List of channels (source-detector pairs)
}