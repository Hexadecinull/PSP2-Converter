#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod converter;
mod error;

use converter::{ConvertOptions, ConvertResult, convert};
use error::ConvertError;
use tauri::Manager;

#[tauri::command]
fn convert_game(opts: ConvertOptions) -> Result<ConvertResult, ConvertError> {
    convert(&opts)
}

#[tauri::command]
fn detect_format(path: String) -> Result<String, ConvertError> {
    use std::path::Path;
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "iso" => Ok("ISO".into()),
        "cso" => Ok("CSO".into()),
        "zso" => Ok("ZSO".into()),
        "pbp" => Ok("PBP".into()),
        other => Err(ConvertError::UnsupportedFormat(format!(".{}", other))),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![convert_game, detect_format])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
