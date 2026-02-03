use crate::models::AppPersistConfig;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

const CONFIG_FILE_NAME: &str = "mc-connect-config.json";

fn get_config_path<R: Runtime>(app_handle: &AppHandle<R>) -> Result<PathBuf, String> {
    app_handle
        .path()
        .app_config_dir()
        .map(|mut path| {
            path.push(CONFIG_FILE_NAME);
            path
        })
        .map_err(|e| format!("設定ディレクトリの取得に失敗しました: {}", e))
}

#[tauri::command]
pub async fn save_config<R: Runtime>(
    app_handle: AppHandle<R>,
    config: AppPersistConfig,
) -> Result<(), String> {
    let path = get_config_path(&app_handle)?;

    // ディレクトリが存在しない場合は作成
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("設定ディレクトリの作成に失敗しました: {}", e))?;
        }
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("設定のシリアライズに失敗しました: {}", e))?;

    fs::write(path, json).map_err(|e| format!("設定ファイルの書き込みに失敗しました: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn load_config<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<Option<AppPersistConfig>, String> {
    let path = get_config_path(&app_handle)?;

    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(path)
        .map_err(|e| format!("設定ファイルの読み込みに失敗しました: {}", e))?;

    let config: AppPersistConfig = serde_json::from_str(&json)
        .map_err(|e| format!("設定のデシリアライズに失敗しました: {}", e))?;

    Ok(Some(config))
}
