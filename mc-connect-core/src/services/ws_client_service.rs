use futures_util::{StreamExt, SinkExt};
use log::{info, error};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use url::Url;
use crate::models::packet::{Message, Command, ConnectPayload, ConnectResponsePayload, Protocol, ServerInfoResponsePayload};

/// WebSocket クライアント側のトンネル動作を管理・提供するサービス。
/// ローカルのポートを Listen し、接続が来るたびにプロキシサーバーへ WS トンネルを張ります。
pub struct WsClientService;

impl WsClientService {
    /// 指定されたローカルポートで待機を開始し、新しい接続をプロキシ経由で転送します。
    pub async fn start_tunnel_with_protocol(bind_addr: String, local_port: u16, ws_url: String, remote_target_port: u16, protocol: Protocol) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 先に WebSocket 接続と初期ハンドシェイクを確認する（疎通確認）
        info!("ゲートウェイへの疎通を確認中: {}...", ws_url);
        match Self::check_connectivity(&ws_url, remote_target_port, protocol.clone()).await {
            Ok(_) => info!("ゲートウェイとのハンドシェイクに成功しました。リスナーを起動します。"),
            Err(e) => {
                error!("ゲートウェイとのハンドシェイクに失敗しました: {}", e);
                return Err(e);
            }
        }

        let listener = TcpListener::bind(format!("{}:{}", bind_addr, local_port)).await?;
        info!("クライアント側の TCP リスナーを {}:{} で開始しました。マイクラ等の接続を待機しています。", bind_addr, local_port);

        loop {
            let (tcp_stream, addr) = listener.accept().await?;
            info!("ローカル接続を検知しました: {}", addr);

            let ws_url_clone = ws_url.clone();
            let proto_clone = protocol.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_tunnel(tcp_stream, ws_url_clone, remote_target_port, proto_clone).await {
                    error!("トンネルセッションが異常終了しました ({}): {}", addr, e);
                }
            });
        }
    }

    /// ゲートウェイへの初期接続とターゲットへの疎通を確認する
    async fn check_connectivity(ws_url: &str, remote_port: u16, protocol: Protocol) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        let connect_content = ConnectPayload {
            protocol,
            port: remote_port,
            compression: None,
        };
        let packet = Message::from_payload(Command::Connect, &connect_content)?;
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
                if !res.success {
                    return Err(res.message.into());
                }
                return Ok(());
            }
        }
        Err("不正なサーバー応答です".into())
    }

    /// サーバーから情報を取得します。
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
                let res: ServerInfoResponsePayload = res_packet.deserialize_payload()?;
                return Ok(res);
            }
        }
        Err("サーバー情報の取得に失敗しました".into())
    }

    /// 個別の TCP 接続を WebSocket 経由でプロキシサーバーへ転送するメインロジック。
    async fn handle_tunnel(tcp_stream: TcpStream, ws_url: String, remote_port: u16, protocol: Protocol) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&ws_url)?;
        
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        let connect_content = ConnectPayload {
            protocol,
            port: remote_port,
            compression: None,
        };
        let packet = Message::from_payload(Command::Connect, &connect_content)?;
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
                if !res.success {
                    return Err(format!("サーバーによって接続が拒否されました: {}", res.message).into());
                }
                info!("サーバーとのハンドシェイクに成功。ターゲット(port:{}) への疎通を確認しました。", remote_port);
            } else {
                return Err("不正な初期パケットを受信しました (期待値: ConnectResponse)".into());
            }
        }

        let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();

        let t2w = async move {
            let mut buf = [0u8; 8192];
            loop {
                match tcp_read.read(&mut buf).await {
                    Ok(0) => break, 
                    Ok(n) => {
                        let packet = Message::new(Command::Data, buf[..n].to_vec());
                        if let Ok(bin) = packet.to_vec() {
                            if let Err(e) = ws_write.send(WsMessage::Binary(bin)).await {
                                error!("WS 送信エラー: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("TCP 読み取り失敗: {}", e);
                        break;
                    }
                }
            }
            if let Ok(msg) = Message::new(Command::Disconnect, vec![]).to_vec() {
                let _ = ws_write.send(WsMessage::Binary(msg)).await;
            }
            let _ = ws_write.close().await;
        };

        let w2t = async move {
            while let Some(msg) = ws_read.next().await {
                match msg {
                    Ok(WsMessage::Binary(bin)) => {
                        if let Ok(packet) = Message::from_slice(&bin) {
                            match packet.command {
                                Command::Data => {
                                    if let Err(e) = tcp_write.write_all(&packet.payload).await {
                                        error!("TCP 書き込み失敗: {}", e);
                                        break;
                                    }
                                }
                                Command::Disconnect => break,
                                _ => {}
                            }
                        }
                    }
                    Ok(WsMessage::Close(_)) => break,
                    Err(e) => {
                        error!("WS 受信エラー: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        };

        tokio::select! {
            _ = t2w => {},
            _ = w2t => {},
        }

        info!("トンネルセッションがクローズされました。");
        Ok(())
    }
}
