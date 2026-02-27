mod state;
mod domain;       // looks for domain/mod.rs
mod io;           // looks for io/mod.rs

use crate::io::snirf_parser::parse_snirf;

#[tauri::command]
fn load_snirf(path: String) -> Result<(), String> {
    let snirf = parse_snirf(&path)?;
    
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![load_snirf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}