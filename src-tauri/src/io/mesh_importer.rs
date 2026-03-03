use std::path::Path;

use nalgebra::{Vector2, Vector3};
use uuid::Uuid;

use crate::domain::mesh::{build_kdtree, Mesh, MeshGeometry, MeshTopology, Vertex};

pub fn load_mesh(filepath: &str) -> Result<Mesh, String> {
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, _materials) = tobj::load_obj(filepath, &load_options)
        .map_err(|e| format!("Failed to load OBJ: {e}"))?;

    let mut positions: Vec<Vector3<f64>> = Vec::new();
    let mut normals: Vec<Vector3<f64>> = Vec::new();
    let mut uvs: Vec<Vector2<f64>> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vertex_offset: u32 = 0;

    for model in &models {
        let mesh = &model.mesh;
        let n = mesh.positions.len() / 3;

        for i in 0..n {
            positions.push(Vector3::new(
                mesh.positions[i * 3] as f64,
                mesh.positions[i * 3 + 1] as f64,
                mesh.positions[i * 3 + 2] as f64,
            ));

            if mesh.normals.len() == mesh.positions.len() {
                normals.push(Vector3::new(
                    mesh.normals[i * 3] as f64,
                    mesh.normals[i * 3 + 1] as f64,
                    mesh.normals[i * 3 + 2] as f64,
                ));
            } else {
                normals.push(Vector3::new(0.0, 1.0, 0.0));
            }

            if mesh.texcoords.len() / 2 == n {
                uvs.push(Vector2::new(
                    mesh.texcoords[i * 2] as f64,
                    mesh.texcoords[i * 2 + 1] as f64,
                ));
            } else {
                uvs.push(Vector2::zeros());
            }
        }

        for &idx in &mesh.indices {
            indices.push(idx + vertex_offset);
        }
        vertex_offset += n as u32;
    }

    let verts: Vec<Vertex> = (0..positions.len())
        .map(|i| Vertex {
            position: positions[i],
            normal: normals[i],
            uv: uvs[i],
        })
        .collect();

    // Build parry3d TriMesh (uses f32)
    let trimesh = if !positions.is_empty() && indices.len() >= 3 {
        let parry_verts: Vec<parry3d::math::Vector> = positions
            .iter()
            .map(|v| parry3d::math::Vector::new(v.x as f32, v.y as f32, v.z as f32))
            .collect();
        let tri_indices: Vec<[u32; 3]> = indices
            .chunks(3)
            .filter(|c| c.len() == 3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        parry3d::shape::TriMesh::new(parry_verts, tri_indices).ok()
    } else {
        None
    };

    let kdtree = build_kdtree(&positions);

    let name = Path::new(filepath)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(Mesh {
        id: Uuid::new_v4(),
        name,
        filepath: filepath.to_string(),
        geometry: MeshGeometry { verts, indices },
        topology: MeshTopology { trimesh, kdtree },
    })
}
