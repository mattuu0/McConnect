use mc_connect_core::models::packet::{Protocol, ServerInfoResponsePayload};
use mc_connect_core::WsClientService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter, Runtime};
use chrono::Local;

#[derive(Default)]
struct AppState {
    tunnel_handle: Option<tokio::task::JoinHandle<()>>,
}

static STATE: Lazy<Arc<Mutex<AppState>>> = Lazy::new(|| Arc::new(Mutex::new(AppState::default())));

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionInfo {
    pub ws_url: String,
    pub local_port: u16,
    pub remote_port: u16,
}

#[derive(Serialize, Clone)]
struct TunnelStatus {
    running: bool,
    message: String,
}

#[derive(Serialize, Clone)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

fn emit_log<R: Runtime>(app: &AppHandle<R>, level: &str, message: String) {
    let timestamp = Local::now().format("%H:%M:%S").to_string();
    let _ = app.emit("log-event", LogEntry {
        timestamp,
        level: level.to_string(),
        message,
    });
}

#[tauri::command]
async fn get_server_info<R: Runtime>(app_handle: AppHandle<R>, ws_url: String) -> Result<ServerInfoResponsePayload, String> {
    emit_log(&app_handle, "INFO", format!("サーバー情報を取得中: {}", ws_url));
    match WsClientService::get_server_info(&ws_url).await {
        Ok(info) => {
            emit_log(&app_handle, "SUCCESS", format!("サーバー情報を取得しました。許可ポート: {}個", info.allowed_ports.len()));
            Ok(info)
        }
        Err(e) => {
            emit_log(&app_handle, "ERROR", format!("サーバー情報の取得に失敗: {}", e));
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn start_tunnel<R: Runtime>(app_handle: AppHandle<R>, info: ConnectionInfo) -> Result<(), String> {
    let mut state = STATE.lock().await;
    
    if let Some(handle) = state.tunnel_handle.take() {
        emit_log(&app_handle, "WARN", "既存のトンネルを停止しています...".into());
        handle.abort();
    }

    let app = app_handle.clone();
    emit_log(&app, "INFO", format!("トンネルを開始します: localhost:{} -> {}", info.local_port, info.ws_url));

    let handle = tokio::spawn(async move {
        let _ = app.emit("tunnel-status", TunnelStatus {
            running: true,
            message: "Establishing tunnel...".into(),
        });

        match WsClientService::start_tunnel_with_protocol(
            info.local_port,
            info.ws_url.clone(),
            info.remote_port,
            Protocol::TCP,
        ).await {
            Ok(_) => {
                emit_log(&app, "INFO", "トンネルセッションが終了しました".into());
                let _ = app.emit("tunnel-status", TunnelStatus {
                    running: false,
                    message: "Tunnel stopped".into(),
                });
            },
            Err(e) => {
                emit_log(&app, "ERROR", format!("トンネルエラー: {}", e));
                let _ = app.emit("tunnel-status", TunnelStatus {
                    running: false,
                    message: format!("Error: {}", e),
                });
            },
        }
    });

    state.tunnel_handle = Some(handle);
    Ok(())
}

#[tauri::command]
async fn stop_tunnel<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    let mut state = STATE.lock().await;
    if let Some(handle) = state.tunnel_handle.take() {
        emit_log(&app_handle, "INFO", "トンネルを手動で停止しました".into());
        handle.abort();
        let _ = app_handle.emit("tunnel-status", TunnelStatus {
            running: false,
            message: "Stopped".into(),
        });
    }
    Ok(())
}

#[tauri::command]
async fn is_tunnel_running() -> bool {
    let state = STATE.lock().await;
    state.tunnel_handle.is_some()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_server_info,
            start_tunnel,
            stop_tunnel,
            is_tunnel_running
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
