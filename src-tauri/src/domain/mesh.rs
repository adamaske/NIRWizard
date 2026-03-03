use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =========================
// Geometry
// =========================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vertex {
    pub position: Vector3<f64>,
    pub normal: Vector3<f64>,
    pub uv: Vector2<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeshGeometry {
    pub verts: Vec<Vertex>,
    pub indices: Vec<u32>,
}

// =========================
// Spatial Topology: BVH, KD-Tree
// =========================

pub struct MeshTopology {
    pub trimesh: Option<parry3d::shape::TriMesh>,
    pub kdtree: kiddo::KdTree<f64, 3>,
}

impl Clone for MeshTopology {
    fn clone(&self) -> Self {
        Self {
            trimesh: self.trimesh.clone(),
            kdtree: self.kdtree.clone(),
        }
    }
}

impl std::fmt::Debug for MeshTopology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshTopology")
            .field("trimesh", &self.trimesh.as_ref().map(|_| "<TriMesh>"))
            .field("kdtree_size", &self.kdtree.size())
            .finish()
    }
}

impl Default for MeshTopology {
    fn default() -> Self {
        Self {
            trimesh: None,
            kdtree: kiddo::KdTree::new(),
        }
    }
}

// =========================
// Mesh
// =========================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mesh {
    pub id: Uuid,
    pub name: String,
    pub filepath: String,
    pub geometry: MeshGeometry,
    #[serde(skip)]
    pub topology: MeshTopology,
}

// =========================
// Helper: build KD-tree from vertex positions
// =========================

pub fn build_kdtree(vertices: &[Vector3<f64>]) -> kiddo::KdTree<f64, 3> {
    let mut tree: kiddo::KdTree<f64, 3> = kiddo::KdTree::new();
    for (i, v) in vertices.iter().enumerate() {
        tree.add(&[v.x, v.y, v.z], i as u64);
    }
    tree
}
