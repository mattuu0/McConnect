use futures_util::{StreamExt, SinkExt};
use log::{info, error};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use url::Url;
use crate::models::packet::{Message, Command, ConnectPayload, ConnectResponsePayload, Protocol};

/// WebSocket クライアント側のトンネルサービス
pub struct WsClientService;

impl WsClientService {
    /// ローカルポートで待機し、接続が来たら WebSocket トンネルを作成する
    /// 
    /// # 引数
    /// * `local_port` - クライアントが待機するローカルポート (例: 25565)
    /// * `ws_url` - プロキシサーバーの URL (例: "ws://localhost:8080/ws")
    /// * `remote_target_port` - サーバー側で最終的に接続するポート
    pub async fn start_tunnel(local_port: u16, ws_url: String, remote_target_port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port)).await?;
        info!("クライアント側の TCP リスナーを 127.0.0.1:{} で開始しました", local_port);

        loop {
            let (tcp_stream, addr) = listener.accept().await?;
            info!("ローカル TCP 接続を受信: {}", addr);

            let ws_url_clone = ws_url.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_tunnel(tcp_stream, ws_url_clone, remote_target_port).await {
                    error!("トンネル処理エラー: {}", e);
                }
            });
        }
    }

    /// 個別の TCP 接続を WebSocket へ橋渡しする
    async fn handle_tunnel(tcp_stream: TcpStream, ws_url: String, remote_port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&ws_url)?;
        
        // 1. プロキシサーバーへの WebSocket 接続
        let (ws_stream, _) = connect_async(url).await?;
        info!("プロキシサーバーへ接続しました: {}", ws_url);
        
        let (mut ws_write, mut ws_read) = ws_stream.split();

        // 2. 初期接続パケット (Connect) の送信
        let connect_content = ConnectPayload {
            protocol: Protocol::TCP,
            port: remote_port,
            compression: None,
        };
        let packet = Message::from_payload(Command::Connect, &connect_content)?;
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        // 3. サーバーからの応答 (ConnectResponse) の待機
        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
                if !res.success {
                    return Err(format!("サーバーが接続を拒否しました: {}", res.message).into());
                }
                info!("サーバーとのハンドシェイクが成功しました");
            } else {
                return Err("不正な初期パケットを受信しました".into());
            }
        }

        // 4. データ転送ループの開始
        let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();

        // --- TCP -> WebSocket 方向 ---
        let t2w = async move {
            let mut buf = [0u8; 8192];
            loop {
                match tcp_read.read(&mut buf).await {
                    Ok(0) => break, // 切断
                    Ok(n) => {
                        // データパケットの作成。バイナリはそのままペイロードへ
                        let packet = Message::new(Command::Data, buf[..n].to_vec());
                        if let Ok(bin) = packet.to_vec() {
                            if let Err(e) = ws_write.send(WsMessage::Binary(bin)).await {
                                error!("WS 送信失敗: {}", e);
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
            // 切断通知
            if let Ok(msg) = Message::new(Command::Disconnect, vec![]).to_vec() {
                let _ = ws_write.send(WsMessage::Binary(msg)).await;
            }
            let _ = ws_write.close().await;
            info!("TCP -> WS 転送タスクを終了しました");
        };

        // --- WebSocket -> TCP 方向 ---
        let w2t = async move {
            while let Some(msg) = ws_read.next().await {
                match msg {
                    Ok(WsMessage::Binary(bin)) => {
                        if let Ok(packet) = Message::from_slice(&bin) {
                            match packet.command {
                                Command::Data => {
                                    // ペイロードをそのまま TCP 側へ書き込む
                                    if let Err(e) = tcp_write.write_all(&packet.payload).await {
                                        error!("TCP 書き込み失敗: {}", e);
                                        break;
                                    }
                                }
                                Command::Disconnect => {
                                    info!("サーバーからの切断通知を受信しました");
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(WsMessage::Close(_)) => break,
                    Err(e) => {
                        error!("WS 受信失敗: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            info!("WS -> TCP 転送タスクを終了しました");
        };

        // いずれかの方向が終了したらトンネルを閉じる
        tokio::select! {
            _ = t2w => {},
            _ = w2t => {},
        }

        Ok(())
    }
}
