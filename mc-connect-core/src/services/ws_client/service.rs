use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use url::Url;

use super::stats::TunnelStats;
use super::tunnel::handle_tunnel;
use crate::encryption::{CryptoError, RsaKeyPair, create_secure_connect_packet};
use crate::models::packet::{
    Command, ConnectResponsePayload, Message, Protocol, ServerInfoResponsePayload,
};

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
        ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
        server_public_key: Arc<RsaKeyPair>,
    ) -> Result<(), CryptoError> {
        info!("ゲートウェイへのセキュア接続を確認中: {}...", ws_url);
        Self::check_connectivity(
            &ws_url,
            remote_target_port,
            protocol.clone(),
            Arc::clone(&server_public_key),
        )
        .await?;

        info!("ゲートウェイとのセキュアハンドシェイクに成功しました。準備完了です。");

        Self::run_tunnel_server(
            bind_addr,
            local_port,
            ws_url,
            remote_target_port,
            protocol,
            stats,
            ping_rx,
            server_public_key,
        )
        .await
    }

    /// [run_tunnel_server]
    /// ローカルでポート待機を開始し、接続ごとにセッションを確立します。
    pub async fn run_tunnel_server(
        bind_addr: String,
        local_port: u16,
        ws_url: String,
        remote_target_port: u16,
        protocol: Protocol,
        stats: Arc<TunnelStats>,
        mut ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
        server_public_key: Arc<RsaKeyPair>,
    ) -> Result<(), CryptoError> {
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

    pub async fn check_connectivity(
        ws_url: &str,
        remote_port: u16,
        protocol: Protocol,
        server_public_key: Arc<RsaKeyPair>,
    ) -> Result<(), CryptoError> {
        info!("ゲートウェイへの接続テストを開始します: {}", ws_url);
        let url = match Url::parse(ws_url) {
            Ok(u) => u,
            Err(e) => {
                error!("URLの解析に失敗しました ({}): {}", ws_url, e);
                return Err(e.into());
            }
        };

        info!("WebSocket 接続中...");
        let (ws_stream, _) = match connect_async(url).await {
            Ok(v) => v,
            Err(e) => {
                error!("WebSocket 接続自体に失敗しました: {}", e);
                return Err(e.into());
            }
        };
        info!("WebSocket 接続に成功しました。セキュアハンドシェイクを試行します...");

        let (mut ws_write, mut ws_read) = ws_stream.split();

        let (secure_context, handshake_packet) =
            match create_secure_connect_packet(protocol, remote_port, server_public_key.as_ref()) {
                Ok(v) => v,
                Err(e) => {
                    error!("ハンドシェイクパケットの生成に失敗: {}", e);
                    return Err(e);
                }
            };

        info!("SecureConnect パケットを送信中...");
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

        info!("ゲートウェイからの応答を待機しています...");
        match ws_read.next().await {
            Some(Ok(msg)) => {
                let bin = msg.into_data();
                info!("応答データを受信しました ({} bytes)。復号中...", bin.len());
                let res_msg = match Message::from_slice(&bin) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("応答メッセージの MessagePack デコードに失敗: {}", e);
                        return Err(e);
                    }
                };
                let res_msg = match secure_context.unseal_message(res_msg) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("応答メッセージの復号に失敗: {}", e);
                        return Err(e);
                    }
                };

                if res_msg.command == Command::ConnectResponse {
                    let res: ConnectResponsePayload = match res_msg.deserialize_payload() {
                        Ok(p) => p,
                        Err(e) => {
                            error!("ConnectResponse ペイロードのデシリアライズに失敗: {}", e);
                            return Err(e);
                        }
                    };
                    if res.success {
                        info!("セキュア接続テストに成功しました。");
                        return Ok(());
                    }
                    error!("サーバーにより接続が拒否されました: {}", res.message);
                    return Err(res.message.into());
                }
                error!("予期しないコマンドを受信しました: {:?}", res_msg.command);
                Err("予期しない応答です".into())
            }
            Some(Err(e)) => {
                error!("WebSocket でエラーが発生しました: {}", e);
                Err(e.into())
            }
            None => {
                error!("ゲートウェイによって接続が閉じられました。");
                Err("接続終了".into())
            }
        }
    }

    pub async fn get_server_info(ws_url: &str) -> Result<ServerInfoResponsePayload, CryptoError> {
        info!("サーバー情報を取得しています: {}", ws_url);
        let url = Url::parse(ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        let packet = Message::new(Command::GetServerInfo, vec![]);
        info!("GetServerInfo パケットを送信中...");
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ServerInfoResponse {
                info!("サーバー情報を正常に取得しました。");
                return Ok(res_packet.deserialize_payload()?);
            }
            error!("想定外の応答です: {:?}", res_packet.command);
        }
        error!("サーバー情報の取得に失敗しました。");
        Err("サーバー情報の取得に失敗しました".into())
    }
}
