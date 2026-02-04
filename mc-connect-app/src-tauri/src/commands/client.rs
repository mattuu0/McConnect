use mc_connect_core::models::packet::{Protocol, ServerInfoResponsePayload};
use mc_connect_core::services::ws_client::TunnelStats;
use mc_connect_core::WsClientService;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::time::{interval, Duration};

use crate::models::{MappingInfo, StatsEvent, TunnelStatus};
use crate::state::{TunnelHandle, STATE};
use crate::utils::emit_log;

#[tauri::command]
pub async fn get_server_info<R: Runtime>(
    app_handle: AppHandle<R>,
    ws_url: String,
) -> Result<ServerInfoResponsePayload, String> {
    emit_log(
        &app_handle,
        "INFO",
        format!("サーバー情報を取得中: {}", ws_url),
    );
    match WsClientService::get_server_info(&ws_url).await {
        Ok(info) => {
            emit_log(
                &app_handle,
                "SUCCESS",
                format!(
                    "サーバー情報を取得しました。許可ポート: {}個",
                    info.allowed_ports.len()
                ),
            );
            Ok(info)
        }
        Err(e) => {
            emit_log(
                &app_handle,
                "ERROR",
                format!("サーバー情報の取得に失敗: {}", e),
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn start_mapping<R: Runtime>(
    app_handle: AppHandle<R>,
    info: MappingInfo,
) -> Result<(), String> {
    emit_log(
        &app_handle,
        "INFO",
        format!("トンネル開始命令を受信: {}", info.id),
    );

    let mut state = STATE.lock().await;

    if let Some(handle) = state.tunnels.remove(&info.id) {
        handle.join_handle.abort();
        handle.stats_handle.abort();
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

    emit_log(
        &app,
        "INFO",
        format!(
            "トンネルを開始します: [{}] {}:{} -> {} (Ping: {}s)",
            mapping_id, bind_addr, local_port, ws_url, ping_interval
        ),
    );

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
        use base64::{engine::general_purpose, Engine as _};
        let der = general_purpose::STANDARD
            .decode(key_str.trim())
            .map_err(|e| format!("公開鍵のデコードに失敗: {}", e))?;
        Arc::new(
            mc_connect_core::encryption::RsaKeyPair::from_public_der(&der)
                .map_err(|e| e.to_string())?,
        )
    } else {
        return Err("公開鍵が設定されていません。".into());
    };

    let stats = Arc::new(TunnelStats::new());
    let (ping_tx, ping_rx) = tokio::sync::mpsc::unbounded_channel();

    let stats_clone = Arc::clone(&stats);
    let app_stats = app.clone();
    let mapping_id_stats = mapping_id.clone();

    // Stats reporting loop
    let stats_handle = tokio::spawn(async move {
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
            stats_clone
                .download_speed
                .store(speed_down, Ordering::Relaxed);

            last_up = cur_up;
            last_down = cur_down;

            let snapshot = stats_clone.get_snapshot();
            if app_stats
                .emit(
                    "tunnel-stats",
                    StatsEvent {
                        id: mapping_id_stats.clone(),
                        stats: snapshot,
                    },
                )
                .is_err()
            {
                break;
            }
        }
    });

    let handle = tokio::spawn(async move {
        // Step 1: Handshake (while frontend is still 'loading')
        match WsClientService::check_connectivity(
            &ws_url,
            remote_port,
            proto.clone(),
            Arc::clone(&server_public_key),
        )
        .await
        {
            Ok(_) => {
                // Step 2: Handshake Success -> Notify UI
                let _ = app.emit(
                    "tunnel-status",
                    TunnelStatus {
                        id: mapping_id.clone(),
                        running: true,
                        message: "接続完了".into(),
                    },
                );

                // Step 3: Run the server loop
                if let Err(e) = WsClientService::run_tunnel_server(
                    bind_addr,
                    local_port,
                    ws_url,
                    remote_port,
                    proto,
                    stats,
                    ping_rx,
                    server_public_key,
                )
                .await
                {
                    emit_log(
                        &app,
                        "ERROR",
                        format!("トンネルエラー [{}]: {}", mapping_id, e),
                    );
                    let _ = app.emit(
                        "tunnel-status",
                        TunnelStatus {
                            id: mapping_id,
                            running: false,
                            message: format!("エラー: {}", e),
                        },
                    );
                } else {
                    emit_log(
                        &app,
                        "INFO",
                        format!("トンネルセッション終了: {}", mapping_id),
                    );
                    let _ = app.emit(
                        "tunnel-status",
                        TunnelStatus {
                            id: mapping_id,
                            running: false,
                            message: "トンネルが停止しました".into(),
                        },
                    );
                }
            }
            Err(e) => {
                emit_log(
                    &app,
                    "ERROR",
                    format!("接続テスト失敗 [{}]: {}", mapping_id, e),
                );
                let _ = app.emit(
                    "tunnel-status",
                    TunnelStatus {
                        id: mapping_id,
                        running: false,
                        message: format!("接続失敗: {}", e),
                    },
                );
            }
        }
    });

    state.tunnels.insert(
        info.id,
        TunnelHandle {
            join_handle: handle,
            stats_handle,
            ping_tx,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn stop_mapping<R: Runtime>(app_handle: AppHandle<R>, id: String) -> Result<(), String> {
    let mut state = STATE.lock().await;
    if let Some(handle) = state.tunnels.remove(&id) {
        emit_log(
            &app_handle,
            "INFO",
            format!("トンネルを手動で停止しました: {}", id),
        );
        handle.join_handle.abort();
        handle.stats_handle.abort();
        let _ = app_handle.emit(
            "tunnel-status",
            TunnelStatus {
                id: id,
                running: false,
                message: "停止しました".into(),
            },
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn is_mapping_running(id: String) -> bool {
    let state = STATE.lock().await;
    state.tunnels.contains_key(&id)
}

#[tauri::command]
pub async fn trigger_ping(id: String) -> Result<(), String> {
    let state = STATE.lock().await;
    if let Some(handle) = state.tunnels.get(&id) {
        let _ = handle.ping_tx.send(());
        Ok(())
    } else {
        Err("Tunnel not running".into())
    }
}
