use crate::domain::mesh::Mesh;
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A 3D transform — position, rotation, scale.
/// Cached matrix is recomputed when components change.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform {
    pub position: na::Vector3<f64>,
    pub rotation: na::Vector3<f64>, // Euler angles in degrees
    pub scale: na::Vector3<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: na::Vector3::zeros(),
            rotation: na::Vector3::zeros(),
            scale: na::Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    /// Compose the full model matrix: Translation * Rotation * Scale
    pub fn matrix(&self) -> na::Matrix4<f64> {
        let translation = na::Translation3::from(self.position).to_homogeneous();

        let rotation = na::Rotation3::from_euler_angles(
            self.rotation.x.to_radians(),
            self.rotation.y.to_radians(),
            self.rotation.z.to_radians(),
        )
        .to_homogeneous();

        let scale = na::Matrix4::new_nonuniform_scaling(&self.scale);

        translation * rotation * scale
    }

    /// Transform a point from local space to world space
    pub fn transform_point(&self, point: &na::Point3<f64>) -> na::Point3<f64> {
        self.matrix().transform_point(point)
    }
}

/// Metadata tags for any scene object
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectMeta {
    pub name: String,
    pub tags: HashMap<String, String>,
}

/// A 3D object in the scene: geometry + transform + display properties.
///
/// This is the Rust-side representation. The frontend (Three.js) maintains
/// its own Mesh/BufferGeometry — we send flat arrays via invoke().
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneObject {
    pub meta: ObjectMeta,
    pub mesh: Mesh,
    pub transform: Transform,
    pub visible: bool,
    pub opacity: f64,
}

impl SceneObject {
    pub fn new(name: impl Into<String>, mesh: Mesh) -> Self {
        let name = name.into();
        let obj = Self {
            meta: ObjectMeta {
                name: name.clone(),
                tags: HashMap::new(),
            },
            mesh,
            transform: Transform::default(),
            visible: true,
            opacity: 1.0,
        };
        println!(
            "[SceneObject] created '{}': {} verts, {} tris",
            name,
            obj.mesh.geometry.verts.len(),
            obj.mesh.geometry.indices.len() / 3,
        );
        obj
    }

    /// Get vertex positions in world space
    pub fn world_positions(&self) -> Vec<na::Point3<f64>> {
        let mat = self.transform.matrix();
        self.mesh
            .geometry
            .verts
            .iter()
            .map(|v| mat.transform_point(&na::Point3::from(v.position)))
            .collect()
    }
}
