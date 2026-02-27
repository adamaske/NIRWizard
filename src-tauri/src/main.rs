mod snirf; // TODO :  What does this do?

use snirf::{parse_snirf, SnirfData}; // What does this do? Is it a include statement

#[tauri::command]
fn load_snirf(path: String) -> Result<SnirfData, String>{
    parse_snirf(&path)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![load_snirf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}