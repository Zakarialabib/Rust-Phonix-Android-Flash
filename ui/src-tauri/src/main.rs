#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = phoenix_gui::run() {
        eprintln!("Fatal error while running tauri application: {}", e);
        std::process::exit(1);
    }
}
