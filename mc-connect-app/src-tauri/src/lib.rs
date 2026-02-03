use mc_connect_core::models::packet::{Protocol, ServerInfoResponsePayload, StatsPayload};
use mc_connect_core::services::ws_client_service::TunnelStats;
use mc_connect_core::WsClientService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};
use chrono::Local;
use tokio::time::{interval, Duration};

struct TunnelHandle {
    join_handle: tokio::task::JoinHandle<()>,
    ping_tx: tokio::sync::mpsc::UnboundedSender<()>,
}

#[derive(Default)]
struct AppState {
    tunnels: HashMap<String, TunnelHandle>,
}

static STATE: Lazy<Arc<Mutex<AppState>>> = Lazy::new(|| Arc::new(Mutex::new(AppState::default())));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingInfo {
    pub id: String,
    pub ws_url: String,
    pub bind_addr: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String,
    pub ping_interval: u64,
    pub public_key: Option<String>,
}

#[derive(Serialize, Clone)]
struct TunnelStatus {
    id: String,
    running: bool,
    message: String,
}

#[derive(Serialize, Clone)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Serialize, Clone)]
struct StatsEvent {
    id: String,
    stats: StatsPayload,
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
async fn start_mapping<R: Runtime>(app_handle: AppHandle<R>, info: MappingInfo) -> Result<(), String> {
    let mut state = STATE.lock().await;
    
    if let Some(handle) = state.tunnels.remove(&info.id) {
        handle.join_handle.abort();
    }

    let app = app_handle.clone();
    let mapping_id = info.id.clone();
    let ws_url = info.ws_url.clone();
    let bind_addr = info.bind_addr.clone();
    let local_port = info.local_port;
    let remote_port = info.remote_port;
    let proto_str = info.protocol.clone();
    let ping_interval = info.ping_interval;
    let public_key_str = info.public_key.clone();

    emit_log(&app, "INFO", format!("トンネルを開始します: [{}] {}:{} -> {} (Ping: {}s)", mapping_id, bind_addr, local_port, ws_url, ping_interval));

    let proto = match proto_str.to_lowercase().as_str() {
        "tcp" => Protocol::TCP,
        "udp" => Protocol::UDP,
        _ => return Err(format!("Unsupported protocol: {}", proto_str)),
    };

    // 公開鍵のパース
    let server_public_key = if let Some(key_str) = public_key_str {
        if key_str.trim().is_empty() {
             return Err("公開鍵が空です。".into());
        }
        use base64::{Engine as _, engine::general_purpose};
        let der = general_purpose::STANDARD.decode(key_str.trim()).map_err(|e| format!("公開鍵のデコードに失敗: {}", e))?;
        Arc::new(mc_connect_core::encryption::RsaKeyPair::from_public_der(&der).map_err(|e| e.to_string())?)
    } else {
        return Err("公開鍵が設定されていません。".into());
    };

    let stats = Arc::new(TunnelStats::new());
    let (ping_tx, ping_rx) = tokio::sync::mpsc::unbounded_channel();
    
    let stats_clone = Arc::clone(&stats);
    let app_stats = app.clone();
    let mapping_id_stats = mapping_id.clone();
    
    // Stats reporting loop
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(1000));
        let mut last_up = 0;
        let mut last_down = 0;
        
        loop {
            interval.tick().await;
            
            let cur_up = stats_clone.upload_total.load(Ordering::Relaxed);
            let cur_down = stats_clone.download_total.load(Ordering::Relaxed);
            
            let speed_up = cur_up.saturating_sub(last_up);
            let speed_down = cur_down.saturating_sub(last_down);
            
            stats_clone.upload_speed.store(speed_up, Ordering::Relaxed);
            stats_clone.download_speed.store(speed_down, Ordering::Relaxed);
            
            last_up = cur_up;
            last_down = cur_down;

            let snapshot = stats_clone.get_snapshot();
            if app_stats.emit("tunnel-stats", StatsEvent { id: mapping_id_stats.clone(), stats: snapshot }).is_err() {
                break;
            }
        }
    });

    let handle = tokio::spawn(async move {
        let _ = app.emit("tunnel-status", TunnelStatus {
            id: mapping_id.clone(),
            running: true,
            message: "Establishing tunnel...".into(),
        });

        match WsClientService::start_tunnel_with_protocol(
            bind_addr,
            local_port,
            ws_url,
            remote_port,
            proto,
            stats,
            ping_rx,
            server_public_key,
        ).await {
            Ok(_) => {
                emit_log(&app, "INFO", format!("トンネルセッション終了: {}", mapping_id));
                let _ = app.emit("tunnel-status", TunnelStatus {
                    id: mapping_id,
                    running: false,
                    message: "Tunnel stopped".into(),
                });
            },
            Err(e) => {
                emit_log(&app, "ERROR", format!("トンネルエラー [{}]: {}", mapping_id, e));
                let _ = app.emit("tunnel-status", TunnelStatus {
                    id: mapping_id,
                    running: false,
                    message: format!("Error: {}", e),
                });
            },
        }
    });

    state.tunnels.insert(info.id, TunnelHandle {
        join_handle: handle,
        ping_tx,
    });
    Ok(())
}

#[tauri::command]
async fn stop_mapping<R: Runtime>(app_handle: AppHandle<R>, id: String) -> Result<(), String> {
    let mut state = STATE.lock().await;
    if let Some(handle) = state.tunnels.remove(&id) {
        emit_log(&app_handle, "INFO", format!("トンネルを手動で停止しました: {}", id));
        handle.join_handle.abort();
        let _ = app_handle.emit("tunnel-status", TunnelStatus {
            id: id,
            running: false,
            message: "Stopped".into(),
        });
    }
    Ok(())
}

#[tauri::command]
async fn is_mapping_running(id: String) -> bool {
    let state = STATE.lock().await;
    state.tunnels.contains_key(&id)
}

#[tauri::command]
async fn trigger_ping(id: String) -> Result<(), String> {
    let state = STATE.lock().await;
    if let Some(handle) = state.tunnels.get(&id) {
        let _ = handle.ping_tx.send(());
        Ok(())
    } else {
        Err("Tunnel not running".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_server_info,
            start_mapping,
            stop_mapping,
            is_mapping_running,
            trigger_ping
        ])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit McConnect", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Open Dashboard", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false) 
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { 
                        button: MouseButton::Left, 
                        button_state: MouseButtonState::Up, 
                        .. 
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
