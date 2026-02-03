use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant, interval};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use url::Url;

use super::stats::TunnelStats;
use crate::encryption::{RsaKeyPair, create_secure_connect_packet};
use crate::models::packet::{Command, ConnectResponsePayload, Message, PingPayload, Protocol};

/// [handle_tunnel]
/// セキュアなトンネル接続を確立し、データの送受信を行うメインロジックです。
pub async fn handle_tunnel(
    tcp_stream: TcpStream,
    ws_url: String,
    remote_port: u16,
    protocol: Protocol,
    stats: Arc<TunnelStats>,
    mut manual_ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
    server_public_key: Arc<RsaKeyPair>, // サーバーの公開鍵
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. WebSocket 接続の開始
    let url = match Url::parse(&ws_url) {
        Ok(u) => u,
        Err(e) => {
            error!("URLの解析に失敗しました: {}", e);
            return Err(e.into());
        }
    };
    info!("WebSocket 接続を開始します: {}", url);
    let (ws_stream, _) = match connect_async(url).await {
        Ok(v) => v,
        Err(e) => {
            error!("WebSocket 接続自体に失敗しました: {}", e);
            return Err(e.into());
        }
    };
    info!("WebSocket 接続が確立されました。");
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // 2. セキュアハンドシェイク (Handshake Phase)
    info!(
        "セキュアハンドシェイクを開始します (Target Port: {}, Protocol: {:?})",
        remote_port, protocol
    );
    let (secure_context, handshake_packet) =
        match create_secure_connect_packet(protocol, remote_port, server_public_key.as_ref()) {
            Ok(v) => v,
            Err(e) => {
                error!("ハンドシェイクパケットの生成に失敗: {}", e);
                return Err(e);
            }
        };

    // ハンドシェイクパケットを送信
    info!("SecureConnect パケットを送信します...");
    let bin = match handshake_packet.to_vec() {
        Ok(b) => b,
        Err(e) => {
            error!("ハンドシェイクパケットのシリアライズに失敗: {}", e);
            return Err(e);
        }
    };

    if let Err(e) = ws_write.send(WsMessage::Binary(bin)).await {
        error!("パケットの送信に失敗しました: {}", e);
        return Err(e.into());
    }

    // サーバーからの応答待ち
    info!("ゲートウェイからの応答を待機中...");
    match ws_read.next().await {
        Some(Ok(msg)) => {
            let bin = msg.into_data();
            let res_packet = match Message::from_slice(&bin) {
                Ok(m) => m,
                Err(e) => {
                    error!("応答メッセージのデコードに失敗: {}", e);
                    return Err(e);
                }
            };

            info!("応答パケットを受信しました。復号を試みます...");
            // 応答パケットを復号
            let res_packet = match secure_context.unseal_message(res_packet) {
                Ok(p) => p,
                Err(e) => {
                    error!("応答メッセージの復号に失敗: {}", e);
                    return Err(e.into());
                }
            };

            if res_packet.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = match res_packet.deserialize_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        error!("ConnectResponse ペイロードのデシリアライズに失敗: {}", e);
                        return Err(e);
                    }
                };
                if !res.success {
                    error!("ゲートウェイが接続を拒否しました: {}", res.message);
                    return Err(
                        format!("Gateway rejected secure connection: {}", res.message).into(),
                    );
                }
                info!("セキュアハンドシェイクに成功しました。暗号化トンネルが有効です。");
            } else {
                error!(
                    "プロトコルエラー: ConnectResponse 以外のパケットを受信しました: {:?}",
                    res_packet.command
                );
                return Err("Protocol error: Expected ConnectResponse after SecureConnect".into());
            }
        }
        Some(Err(e)) => {
            error!("WebSocket でエラーが発生しました: {}", e);
            return Err(e.into());
        }
        None => {
            error!("ハンドシェイク中にサーバーによって接続が閉じられました。");
            return Err("Connection closed by server during handshake".into());
        }
    }

    // --- 以降、すべての通信は secure_context を通じて暗号化されます ---

    let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();
    let (internal_tx, mut internal_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // タスク 1: TCP -> WS (Upload)
    let stats_up = Arc::clone(&stats);
    let itx_up = internal_tx.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 8192];
        loop {
            match tcp_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    stats_up
                        .upload_total
                        .fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                    // データを Data パケットとして送信キューへ
                    if itx_up
                        .send(Message::new(Command::Data, buf[..n].to_vec()))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    error!("TCP read error: {}", e);
                    break;
                }
            }
        }
        let _ = itx_up.send(Message::new(Command::Disconnect, vec![]));
    });

    // タスク 2: 定期 Ping 送信
    let itx_ping = internal_tx.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let ping = PingPayload {
                timestamp: Instant::now().elapsed().as_millis() as u64,
            };
            if let Ok(p) = Message::from_payload(Command::Ping, &ping) {
                if itx_ping.send(p).is_err() {
                    break;
                }
            }
        }
    });

    // メインループ: 通信の調停
    loop {
        tokio::select! {
            // [受信] ゲートウェイ(WS) からの暗号化パケットを受信
            Some(msg) = ws_read.next() => {
                match msg {
                    Ok(ws_msg) => {
                        let bin = ws_msg.into_data();
                        let encrypted_packet = Message::from_slice(&bin)?;

                        // パケットを復号
                        let packet = match secure_context.unseal_message(encrypted_packet) {
                            Ok(p) => p,
                            Err(e) => {
                                error!("Packet decryption error: {}. Closing tunnel.", e);
                                break;
                            }
                        };

                        match packet.command {
                            Command::Data => {
                                stats.download_total.fetch_add(packet.payload.len() as u64, std::sync::atomic::Ordering::Relaxed);
                                if let Err(e) = tcp_write.write_all(&packet.payload).await {
                                    error!("TCP write error: {}", e);
                                    break;
                                }
                            }
                            Command::Pong => {
                                if let Ok(payload) = packet.deserialize_payload::<PingPayload>() {
                                    let rtt = (Instant::now().elapsed().as_millis() as u64).saturating_sub(payload.timestamp);
                                    stats.last_rtt_ms.store(rtt, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                            Command::Disconnect => {
                                info!("Gateway requested disconnection from secure tunnel.");
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("WebSocket read error: {}", e);
                        break;
                    }
                }
            }

            // [送信] 内部タスクからの送信要求を受け取り、暗号化して WS へ送る
            Some(mut packet) = internal_rx.recv() => {
                // パケットを暗号化
                packet = match secure_context.seal_message(packet) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Packet encryption error: {}", e);
                        break;
                    }
                };

                if let Ok(bin) = packet.to_vec() {
                    if let Err(e) = ws_write.send(WsMessage::Binary(bin)).await {
                        error!("WebSocket send error: {}", e);
                        break;
                    }
                }
                if packet.command == Command::Disconnect { break; }
            }

            // [手動Ping] 暗号化して送信
            Some(_) = manual_ping_rx.recv() => {
                let ping = PingPayload { timestamp: Instant::now().elapsed().as_millis() as u64 };
                if let Ok(p) = Message::from_payload(Command::Ping, &ping) {
                    if let Ok(sealed) = secure_context.seal_message(p) {
                        if let Ok(bin) = sealed.to_vec() {
                            let _ = ws_write.send(WsMessage::Binary(bin)).await;
                        }
                    }
                }
            }
        }
    }

    info!("Secure tunnel session closed.");
    Ok(())
}
