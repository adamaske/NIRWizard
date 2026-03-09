pub mod snirf;
pub use snirf::*;
pub mod mesh;
pub mod nirs_view; // ← new
pub use mesh::{Mesh, MeshGeometry, MeshTopology, Vertex};
pub use nirs_view::{ChannelView, DataKind, HemoType, NirsView, SignalKind};
pub mod scene;
pub use scene::{ObjectMeta, SceneObject, Transform};
pub mod probe;
pub use probe::{ChannelConnection, Optode3D, OptodeLayout, ProbeDisplaySettings};
pub mod anatomy;
pub use anatomy::SubjectAnatomy;
pub mod voxel;
pub use voxel::VoxelVolume;
