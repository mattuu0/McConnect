use mc_connect_core::models::packet::{AllowedPort, Protocol, ServerInfoResponsePayload};
use mc_connect_core::services::ws_client::TunnelStats;
use mc_connect_core::WsClientService;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::time::{interval, Duration};

use crate::models::{MappingInfo, StartServerConfig, StatsEvent, TunnelStatus};
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
        let _ = app.emit(
            "tunnel-status",
            TunnelStatus {
                id: mapping_id.clone(),
                running: true,
                message: "Establishing tunnel...".into(),
            },
        );

        match WsClientService::start_tunnel_with_protocol(
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
            Ok(_) => {
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
                        message: "Tunnel stopped".into(),
                    },
                );
            }
            Err(e) => {
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
                        message: format!("Error: {}", e),
                    },
                );
            }
        }
    });

    state.tunnels.insert(
        info.id,
        TunnelHandle {
            join_handle: handle,
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
        let _ = app_handle.emit(
            "tunnel-status",
            TunnelStatus {
                id: id,
                running: false,
                message: "Stopped".into(),
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
pub async fn generate_server_keys() -> Result<(String, String), String> {
    use base64::{engine::general_purpose, Engine as _};
    use mc_connect_core::encryption::{KeyGenerator, RsaKeyGenerator};

    let gen = RsaKeyGenerator { bits: 2048 };
    let pair = gen.generate().map_err(|e| e.to_string())?;

    let priv_b64 = general_purpose::STANDARD.encode(pair.private_key_bytes());
    let pub_b64 = general_purpose::STANDARD.encode(pair.public_key_bytes());

    Ok((priv_b64, pub_b64))
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

#[tauri::command]
pub async fn start_server<R: Runtime>(
    app_handle: AppHandle<R>,
    config: StartServerConfig,
) -> Result<(), String> {
    let port = config.port;
    let allowed_ports = config.allowed_ports;
    let private_key_b64 = config.private_key_b64;
    let state = STATE.lock().await;
    if state.server_handle.is_some() {
        return Err("Server is already running".into());
    }

    use base64::{engine::general_purpose, Engine as _};
    use mc_connect_core::encryption::RsaKeyPair;
    use mc_connect_core::models::packet::Protocol as Proto;

    let der = general_purpose::STANDARD
        .decode(private_key_b64.trim())
        .map_err(|e| format!("秘密鍵のデコードに失敗: {}", e))?;
    let key_pair = Arc::new(RsaKeyPair::from_private_der(&der).map_err(|e| e.to_string())?);

    let mut ports = Vec::new();
    for (p, proto_str) in allowed_ports {
        let protocol = match proto_str.to_lowercase().as_str() {
            "tcp" => Proto::TCP,
            "udp" => Proto::UDP,
            _ => continue,
        };
        ports.push(AllowedPort { port: p, protocol });
    }

    let app = app_handle.clone();
    emit_log(
        &app,
        "INFO",
        format!("サーバーを起動します (Port: {})", port),
    );

    let _handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            match mc_connect_core::start_server("0.0.0.0", port, ports, key_pair).await {
                Ok(_) => emit_log(&app, "INFO", "サーバーが終了しました".into()),
                Err(e) => emit_log(&app, "ERROR", format!("サーバーエラー: {}", e)),
            }
        });
    });

    // For now we don't store the JoinHandle for std::thread, but we should fix this if needed.
    // Ok(())
    Ok(())
}

#[tauri::command]
pub async fn stop_server<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    let mut state = STATE.lock().await;
    if let Some(handle) = state.server_handle.take() {
        handle.abort();
        emit_log(&app_handle, "INFO", "サーバーを停止しました".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn is_server_running() -> bool {
    let state = STATE.lock().await;
    state.server_handle.is_some()
}
