pub mod alerts;
pub mod codex_log;
pub mod scanner;
pub mod usage_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
