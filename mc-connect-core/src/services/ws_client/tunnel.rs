use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use futures_util::{StreamExt, SinkExt};
use url::Url;
use log::{error, info};
use tokio::time::{interval, Duration, Instant};

use crate::models::packet::{Message, Command, ConnectResponsePayload, Protocol, PingPayload};
use crate::encryption::{RsaKeyPair, create_secure_connect_packet};
use super::stats::TunnelStats;

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
    let url = Url::parse(&ws_url)?;
    let (ws_stream, _) = connect_async(url).await?;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // 2. セキュアハンドシェイク (Handshake Phase)
    // 共通鍵(AES)を生成し、それをサーバーの公開鍵(RSA)で暗号化したパケットを作成します。
    let (secure_context, handshake_packet) = create_secure_connect_packet(
        protocol, 
        remote_port, 
        server_public_key.as_ref()
    )?;

    // ハンドシェイクパケットを送信（このパケット自体は暗号化されていないコンテナで送る）
    ws_write.send(WsMessage::Binary(handshake_packet.to_vec()?)).await?;

    // サーバーからの応答待ち（30秒タイムアウトは WebSocket レイヤーやサーバー側で管理）
    if let Some(msg) = ws_read.next().await {
        let bin = msg?.into_data();
        let res_packet = Message::from_slice(&bin)?;
        
        // 応答パケットを復号
        let res_packet = secure_context.unseal_message(res_packet)?;

        if res_packet.command == Command::ConnectResponse {
            let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
            if !res.success {
                return Err(format!("Gateway rejected secure connection: {}", res.message).into());
            }
            info!("Secure handshake successful. Symmetric encryption is now active.");
        } else {
            return Err("Protocol error: Expected ConnectResponse after SecureConnect".into());
        }
    } else {
        return Err("Connection closed by server during handshake".into());
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
                    stats_up.upload_total.fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                    // データを Data パケットとして送信キューへ
                    if itx_up.send(Message::new(Command::Data, buf[..n].to_vec())).is_err() {
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
            let ping = PingPayload { timestamp: Instant::now().elapsed().as_millis() as u64 };
            if let Ok(p) = Message::from_payload(Command::Ping, &ping) {
                if itx_ping.send(p).is_err() { break; }
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
