use std::path::PathBuf;

const APP_FOLDER: &str = "RustVoiceStudio";

pub fn app_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(std::env::temp_dir)
        .join(APP_FOLDER)
}

pub fn config_file() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn history_file() -> PathBuf {
    app_data_dir().join("history.json")
}

pub fn default_export_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| app_data_dir())
        .join(APP_FOLDER)
        .join("Exports")
}

pub fn ensure_app_directories() -> Result<(), String> {
    std::fs::create_dir_all(app_data_dir())
        .map_err(|error| format!("Não foi possível criar a pasta de dados: {error}"))?;
    std::fs::create_dir_all(default_export_dir())
        .map_err(|error| format!("Não foi possível criar a pasta de exportação: {error}"))?;
    Ok(())
}
