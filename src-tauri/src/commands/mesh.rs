use crate::io::mesh_importer;

#[tauri::command]
pub fn load_mesh_obj(path: String) -> Result<(), String> {
    let mesh = mesh_importer::load_mesh(&path)?;
    println!("=== Loaded Mesh ===");
    println!("Name:     {}", mesh.name);
    println!("Path:     {}", mesh.filepath);
    println!("Vertices: {}", mesh.geometry.verts.len());
    println!("Indices:  {}", mesh.geometry.indices.len());
    println!("Triangles:{}", mesh.geometry.indices.len() / 3);
    println!("Topology: {:?}", mesh.topology);
    println!("===================");
    Ok(())
}
