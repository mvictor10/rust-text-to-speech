use crate::domain::models::{AppSettings, HistoryEntry};
use crate::services::paths::{config_file, ensure_app_directories, history_file};
use std::fs;

pub fn load_settings() -> AppSettings {
    let _ = ensure_app_directories();
    fs::read(config_file())
        .ok()
        .and_then(|bytes| serde_json::from_slice::<AppSettings>(&bytes).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    ensure_app_directories()?;
    let data = serde_json::to_vec_pretty(settings)
        .map_err(|error| format!("Falha ao serializar as configurações: {error}"))?;
    fs::write(config_file(), data)
        .map_err(|error| format!("Falha ao salvar as configurações: {error}"))
}

pub fn load_history() -> Vec<HistoryEntry> {
    let _ = ensure_app_directories();
    fs::read(history_file())
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Vec<HistoryEntry>>(&bytes).ok())
        .unwrap_or_default()
}

pub fn save_history(entries: &[HistoryEntry]) -> Result<(), String> {
    ensure_app_directories()?;
    let data = serde_json::to_vec_pretty(entries)
        .map_err(|error| format!("Falha ao serializar o histórico: {error}"))?;
    fs::write(history_file(), data)
        .map_err(|error| format!("Falha ao salvar o histórico: {error}"))
}
