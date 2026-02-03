use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use futures_util::{StreamExt, SinkExt};
use url::Url;
use log::{info, error};

use crate::models::packet::{Message, Command, ConnectResponsePayload, Protocol, ServerInfoResponsePayload};
use crate::encryption::{RsaKeyPair, create_secure_connect_packet, CryptoError};
use super::stats::TunnelStats;
use super::tunnel::handle_tunnel;

/// [WsClientService]
/// WebSocket クライアント側のトンネル管理サービスです。
pub struct WsClientService;

impl WsClientService {
    /// [start_tunnel_with_protocol]
    /// ローカルでポート待機を開始し、接続ごとにセッションを確立します。
    pub async fn start_tunnel_with_protocol(
        bind_addr: String, 
        local_port: u16, 
        ws_url: String, 
        remote_target_port: u16, 
        protocol: Protocol,
        stats: Arc<TunnelStats>,
        mut ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
        server_public_key: Arc<RsaKeyPair>,
    ) -> Result<(), CryptoError> {
        
        info!("ゲートウェイへのセキュア接続を確認中: {}...", ws_url);
        match Self::check_connectivity(&ws_url, remote_target_port, protocol.clone(), Arc::clone(&server_public_key)).await {
            Ok(_) => info!("ゲートウェイとのセキュアハンドシェイクに成功しました。準備完了です。"),
            Err(e) => {
                error!("ゲートウェイとの接続に失敗しました: {}", e);
                return Err(e);
            }
        }

        let listener = TcpListener::bind(format!("{}:{}", bind_addr, local_port)).await?;
        info!("TCP リスナーを開始しました: {}:{}", bind_addr, local_port);

        let mut join_set = tokio::task::JoinSet::new();
        let mut session_ping_txs = Vec::<tokio::sync::mpsc::UnboundedSender<()>>::new();

        loop {
            tokio::select! {
                conn = listener.accept() => {
                    match conn {
                        Ok((tcp_stream, addr)) => {
                            info!("新規ローカル接続: {}", addr);
                            
                            let ws_url_clone = ws_url.clone();
                            let proto_clone = protocol.clone();
                            let stats_clone = Arc::clone(&stats);
                            let key_clone = Arc::clone(&server_public_key);
                            
                            let (session_ping_tx, session_ping_rx) = tokio::sync::mpsc::unbounded_channel();
                            session_ping_txs.push(session_ping_tx);

                            join_set.spawn(async move {
                                if let Err(e) = handle_tunnel(
                                    tcp_stream, 
                                    ws_url_clone, 
                                    remote_target_port, 
                                    proto_clone, 
                                    stats_clone, 
                                    session_ping_rx,
                                    key_clone
                                ).await {
                                    error!("トンネルセッションが異常終了しました ({}): {}", addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("TCP accept エラー: {}", e);
                            break;
                        }
                    }
                }
                Some(_) = ping_rx.recv() => {
                    session_ping_txs.retain(|tx| tx.send(()).is_ok());
                }
                _ = join_set.join_next(), if !join_set.is_empty() => {}
            }
        }
        Ok(())
    }

    async fn check_connectivity(
        ws_url: &str, 
        remote_port: u16, 
        protocol: Protocol,
        server_public_key: Arc<RsaKeyPair>,
    ) -> Result<(), CryptoError> {
        let url = Url::parse(ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        let (secure_context, handshake_packet) = create_secure_connect_packet(
            protocol, 
            remote_port, 
            server_public_key.as_ref()
        )?;

        ws_write.send(WsMessage::Binary(handshake_packet.to_vec()?)).await?;

        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_msg = Message::from_slice(&bin)?;
            let res_msg = secure_context.unseal_message(res_msg)?;
            
            if res_msg.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = res_msg.deserialize_payload()?;
                if res.success { return Ok(()); }
                return Err(res.message.into());
            }
        }
        Err("有効なセキュア応答が得られませんでした。".into())
    }

    pub async fn get_server_info(ws_url: &str) -> Result<ServerInfoResponsePayload, CryptoError> {
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
        Err("サーバー情報の取得に失敗しました".into())
    }
}
