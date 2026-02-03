use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use futures_util::{StreamExt, SinkExt};
use url::Url;
use log::{error, info};
use tokio::time::{interval, Duration, Instant};

use crate::models::packet::{Message, Command, ConnectPayload, ConnectResponsePayload, Protocol, PingPayload};
use super::stats::TunnelStats;

/// [handle_tunnel]
/// 個別の接続セッション（1つのTCP接続からゲートウェイへのトンネル）を処理するメインロジックです。
/// 
/// この関数は以下の役割を担います：
/// 1. ゲートウェイへの WebSocket 接続の確立
/// 2. プロトコル・ポート指定によるハンドシェイク
/// 3. TCP -> WS（アップロード）の転送タスク管理
/// 4. 定期的な Ping 送信による死活監視と RTT 計測
/// 5. WS -> TCP（ダウンロード）の転送、および各種制御メッセージの処理
pub async fn handle_tunnel(
    tcp_stream: TcpStream, 
    ws_url: String, 
    remote_port: u16, 
    protocol: Protocol,
    stats: Arc<TunnelStats>,
    mut manual_ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // --- 準備フェーズ: WebSocket 接続 ---
    let url = Url::parse(&ws_url)?;
    // 指定されたURL（ゲートウェイ）に対して WebSocket 接続を開始
    let (ws_stream, _) = connect_async(url).await?;
    // 書き込み用(Sink)と読み取り用(Stream)に分割
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // --- ハンドシェイクフェーズ ---
    // ゲートウェイに対して「どのポートにどのプロトコルで繋いでほしいか」を通知
    let connect_content = ConnectPayload { protocol, port: remote_port, compression: None };
    let packet = Message::from_payload(Command::Connect, &connect_content)?;
    ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

    // ゲートウェイからの応答待ち
    if let Some(msg) = ws_read.next().await {
        let bin = msg?.into_data();
        let res_packet = Message::from_slice(&bin)?;
        if res_packet.command == Command::ConnectResponse {
            let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
            if !res.success {
                // ゲートウェイ側で接続が拒否（許可されていないポートなど）された場合
                return Err(format!("Gateway rejected connection: {}", res.message).into());
            }
        } else {
            return Err("Protocol error: Unexpected packet received during handshake".into());
        }
    }

    // TCPストリームを読み取り側と書き込み側に分割
    let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();
    
    // 他の非同期タスクから WebSocket の書き込み側へデータを送るための内部チャネル
    let (internal_tx, mut internal_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // --- タスク 1: TCP -> WS (Upload) ---
    // ローカルアプリ（Minecraft等）からのデータを読み取り、WebSocket経由で送信します。
    let stats_up = Arc::clone(&stats);
    let itx_up = internal_tx.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 8192]; // 8KB バッファ
        loop {
            match tcp_read.read(&mut buf).await {
                Ok(0) => break, // TCP接続が正常に閉じられた
                Ok(n) => {
                    // 統計情報を更新（アップロード量加算）
                    stats_up.upload_total.fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                    // データを MessagePack パケットとして送信キューへ
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
        // TCPが切れたらゲートウェイにも切断を知らせる
        let _ = itx_up.send(Message::new(Command::Disconnect, vec![]));
    });

    // --- タスク 2: 定期 Ping 送信 ---
    // 5秒おきに Ping を送り、セッションの維持（タイムアウト防止）と遅延計測を行います。
    let itx_ping = internal_tx.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            // 現在のタイムスタンプを載せて送信（Pong で戻ってきた時に RTT を計算する）
            let ping = PingPayload { timestamp: Instant::now().elapsed().as_millis() as u64 };
            if let Ok(p) = Message::from_payload(Command::Ping, &ping) {
                if itx_ping.send(p).is_err() { break; }
            }
        }
    });

    // --- メインループ: 各種イベントの調停 ---
    // WebSocketからのダウンロード、内部タスクからの送信、手動Ping要求を待ち受けます。
    loop {
        tokio::select! {
            // [ルートA] ゲートウェイ(WS) からの着信データを処理
            Some(msg) = ws_read.next() => {
                match msg {
                    Ok(ws_msg) => {
                        let bin = ws_msg.into_data();
                        let packet = Message::from_slice(&bin)?;
                        match packet.command {
                            // 通常データ: TCP側（ローカルアプリ）へ書き込み
                            Command::Data => {
                                stats.download_total.fetch_add(packet.payload.len() as u64, std::sync::atomic::Ordering::Relaxed);
                                if let Err(e) = tcp_write.write_all(&packet.payload).await {
                                    error!("TCP write error: {}", e);
                                    break;
                                }
                            }
                            // 遅延計測の応答: 統計情報を更新
                            Command::Pong => {
                                if let Ok(payload) = packet.deserialize_payload::<PingPayload>() {
                                    let rtt = (Instant::now().elapsed().as_millis() as u64).saturating_sub(payload.timestamp);
                                    stats.last_rtt_ms.store(rtt, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                            // ゲートウェイ側からの切断要求
                            Command::Disconnect => {
                                info!("Gateway requested disconnection.");
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

            // [ルートB] 内部のUploadタスクやPingタスクからの送信要求を処理
            Some(packet) = internal_rx.recv() => {
                if let Ok(bin) = packet.to_vec() {
                    if let Err(e) = ws_write.send(WsMessage::Binary(bin)).await {
                        error!("WebSocket send error: {}", e);
                        break;
                    }
                }
                // 切断パケットを送信した場合はループを終了
                if packet.command == Command::Disconnect { break; }
            }

            // [ルートC] UIなど外部からの明示的な Ping 要求
            Some(_) = manual_ping_rx.recv() => {
                let ping = PingPayload { timestamp: Instant::now().elapsed().as_millis() as u64 };
                if let Ok(p) = Message::from_payload(Command::Ping, &ping) {
                    if let Ok(bin) = p.to_vec() {
                        let _ = ws_write.send(WsMessage::Binary(bin)).await;
                    }
                }
            }
        }
    }
    
    info!("Tunnel session closed.");
    Ok(())
}
