use mc_connect_core::encryption::RsaKeyPair;
use mc_connect_core::models::packet::{AllowedPort, Protocol as Proto};
use std::sync::Arc;
use tauri::{AppHandle, Runtime};

use crate::models::StartServerConfig;
use crate::state::STATE;
use crate::utils::emit_log;

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
