use super::{read_app_config, write_app_config};
use crate::types::{AppConfig, AppConfigResponse, SaveAppConfigRequest};

#[tauri::command]
pub fn get_app_config() -> Result<AppConfigResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    Ok(AppConfigResponse {
        config: read_app_config(&home),
    })
}

#[tauri::command]
pub fn save_app_config(request: SaveAppConfigRequest) -> Result<AppConfigResponse, String> {
    if !matches!(request.default_version_strategy.as_str(), "manual" | "latest" | "stable") {
        return Err("Invalid default version strategy".to_string());
    }

    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let config = AppConfig {
        default_version_strategy: request.default_version_strategy,
    };
    write_app_config(&home, &config)?;
    Ok(AppConfigResponse { config })
}
