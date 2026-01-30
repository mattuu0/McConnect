use futures_util::{StreamExt, SinkExt};
use log::{info, error};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use url::Url;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{interval, Duration, Instant};
use crate::models::packet::{Message, Command, ConnectPayload, ConnectResponsePayload, Protocol, ServerInfoResponsePayload, StatsPayload, PingPayload};

/// トンネルの統計情報を保持する構造体
#[derive(Debug, Default)]
pub struct TunnelStats {
    pub upload_total: AtomicU64,
    pub download_total: AtomicU64,
    pub upload_speed: AtomicU64,
    pub download_speed: AtomicU64,
    pub last_rtt_ms: AtomicU64,
}

impl TunnelStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_snapshot(&self) -> StatsPayload {
        StatsPayload {
            upload_total: self.upload_total.load(Ordering::Relaxed),
            download_total: self.download_total.load(Ordering::Relaxed),
            upload_speed: self.upload_speed.load(Ordering::Relaxed),
            download_speed: self.download_speed.load(Ordering::Relaxed),
            rtt_ms: Some(self.last_rtt_ms.load(Ordering::Relaxed)),
        }
    }
}

/// WebSocket クライアント側のトンネル動作を管理・提供するサービス。
pub struct WsClientService;

impl WsClientService {
    /// 指定されたローカルポートで待機を開始し、新しい接続をプロキシ経由で転送します。
    pub async fn start_tunnel_with_protocol(
        bind_addr: String, 
        local_port: u16, 
        ws_url: String, 
        remote_target_port: u16, 
        protocol: Protocol,
        stats: Arc<TunnelStats>,
        mut ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
        ping_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        info!("ゲートウェイへの疎通を確認中: {}...", ws_url);
        match Self::check_connectivity(&ws_url, remote_target_port, protocol.clone()).await {
            Ok(_) => info!("ゲートウェイとのハンドシェイクに成功しました。"),
            Err(e) => {
                error!("ゲートウェイとのハンドシェイクに失敗しました: {}", e);
                return Err(e);
            }
        }

        let listener = TcpListener::bind(format!("{}:{}", bind_addr, local_port)).await?;
        info!("TCP リスナーを {}:{} で開始しました。", bind_addr, local_port);

        let mut join_set = tokio::task::JoinSet::new();
        // セッションごとの Ping 用送信口を管理
        let mut session_ping_txs = Vec::<tokio::sync::mpsc::UnboundedSender<()>>::new();

        loop {
            tokio::select! {
                // 新しい接続を受け付け
                conn = listener.accept() => {
                    match conn {
                        Ok((tcp_stream, addr)) => {
                            info!("接続を検知: {}", addr);
                            let ws_url_clone = ws_url.clone();
                            let proto_clone = protocol.clone();
                            let stats_clone = Arc::clone(&stats);
                            
                            let (session_ping_tx, session_ping_rx) = tokio::sync::mpsc::unbounded_channel();
                            session_ping_txs.push(session_ping_tx);

                            join_set.spawn(async move {
                                if let Err(e) = Self::handle_tunnel(tcp_stream, ws_url_clone, remote_target_port, proto_clone, stats_clone, session_ping_rx, ping_interval_secs).await {
                                    error!("セッション異常終了 ({}): {}", addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Accept エラー: {}", e);
                            break;
                        }
                    }
                }
                // 外部（AppStateなど）からの Ping 要求を受け取り、全セッションに波及させる
                Some(_) = ping_rx.recv() => {
                    session_ping_txs.retain(|tx| tx.send(()).is_ok());
                }
                // 終了したタスクを回収
                _ = join_set.join_next(), if !join_set.is_empty() => {}
            }
        }
        Ok(())
    }

    /// 疎通確認
    async fn check_connectivity(ws_url: &str, remote_port: u16, protocol: Protocol) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        let connect_content = ConnectPayload { protocol, port: remote_port, compression: None };
        let packet = Message::from_payload(Command::Connect, &connect_content)?;
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
                if !res.success { return Err(res.message.into()); }
                return Ok(());
            }
        }
        Err("不正な応答です".into())
    }

    /// サーバー情報取得
    pub async fn get_server_info(ws_url: &str) -> Result<ServerInfoResponsePayload, Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();
        let packet = Message::new(Command::GetServerInfo, vec![]);
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ServerInfoResponse {
                return Ok(res_packet.deserialize_payload()?);
            }
        }
        Err("失敗しました".into())
    }

    /// 個別トンネル
    async fn handle_tunnel(
        tcp_stream: TcpStream, 
        ws_url: String, 
        remote_port: u16, 
        protocol: Protocol,
        stats: Arc<TunnelStats>,
        mut manual_ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
        ping_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        // ハンドシェイク
        let connect_content = ConnectPayload { protocol, port: remote_port, compression: None };
        let packet = Message::from_payload(Command::Connect, &connect_content)?;
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
                if !res.success { return Err(res.message.into()); }
            } else { return Err("不正なパケットです".into()); }
        }

        let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();
        let (internal_tx, mut internal_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

        // TCP -> WS (Upload)
        let stats_up = Arc::clone(&stats);
        let itx_up = internal_tx.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            while let Ok(n) = tcp_read.read(&mut buf).await {
                if n == 0 { break; }
                stats_up.upload_total.fetch_add(n as u64, Ordering::Relaxed);
                if itx_up.send(Message::new(Command::Data, buf[..n].to_vec())).is_err() { break; }
            }
            let _ = itx_up.send(Message::new(Command::Disconnect, vec![]));
        });

        // 自動定期 Ping & 統計パケット
        let itx_ping = internal_tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(ping_interval_secs));
            loop {
                interval.tick().await;
                let ping = PingPayload { timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64 };
                if let Ok(p) = Message::from_payload(Command::Ping, &ping) {
                    if itx_ping.send(p).is_err() { break; }
                }
            }
        });

        // メインループ
        loop {
            tokio::select! {
                // WS -> TCP (Download) & Ping Response
                Some(msg) = ws_read.next() => {
                    let bin = msg?.into_data();
                    let packet = Message::from_slice(&bin)?;
                    match packet.command {
                        Command::Data => {
                            stats.download_total.fetch_add(packet.payload.len() as u64, Ordering::Relaxed);
                            tcp_write.write_all(&packet.payload).await?;
                        }
                        Command::Pong => {
                            if let Ok(payload) = packet.deserialize_payload::<PingPayload>() {
                                let rtt = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64).saturating_sub(payload.timestamp);
                                stats.last_rtt_ms.store(rtt, Ordering::Relaxed);
                            }
                        }
                        Command::Disconnect => break,
                        _ => {}
                    }
                }
                // 内部送信。手動 Ping もここ経由で WS へ。
                Some(packet) = internal_rx.recv() => {
                    ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;
                    if packet.command == Command::Disconnect { break; }
                }
                // 外部からの手動 Ping 要求
                Some(_) = manual_ping_rx.recv() => {
                    let ping = PingPayload { timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64 };
                    if let Ok(p) = Message::from_payload(Command::Ping, &ping) {
                        ws_write.send(WsMessage::Binary(p.to_vec()?)).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
