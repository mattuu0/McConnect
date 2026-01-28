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
    /// 
    /// # 引数
    /// * `local_port` - クライアント側（手元の PC 等）で Listen するポート (例: 25565)
    /// * `ws_url` - プロキシサーバーの WebSocket エンドポイント URL
    /// * `remote_target_port` - 最終的にサーバー側で接続してほしいポート
    pub async fn start_tunnel(local_port: u16, ws_url: String, remote_target_port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::start_tunnel_with_protocol(local_port, ws_url, remote_target_port, Protocol::TCP).await
    }

    /// 指定されたローカルポートで待機を開始し、新しい接続をプロキシ経由で転送します。
    pub async fn start_tunnel_with_protocol(local_port: u16, ws_url: String, remote_target_port: u16, protocol: Protocol) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port)).await?;
        info!("クライアント側の TCP リスナーを 127.0.0.1:{} で開始しました。マイクラ等の接続を待機しています。", local_port);

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
        
        // --- 1. プロキシサーバーへの WebSocket 接続の確立 ---
        let (ws_stream, _) = connect_async(url).await?;
        info!("プロキシサーバー(WS)への接続に成功しました: {}", ws_url);
        
        // 送信(write)と受信(read)に分離
        let (mut ws_write, mut ws_read) = ws_stream.split();

        // --- 2. プロトコル固有の初期ハンドシェイク (Connect パケット) ---
        // サーバー側に、どのポートへ接続してほしいかを伝えます。
        let connect_content = ConnectPayload {
            protocol,
            port: remote_port,
            compression: None, // 圧縮プロトコルは将来用
        };
        let packet = Message::from_payload(Command::Connect, &connect_content)?;
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        // --- 3. サーバーからの応答待機 ---
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

        // --- 4. 双方向データ転送ループの開始 ---
        // TCP ストリームを読み取り用と書き込み用に分割
        let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();

        // [タスク A] TCP(Local) -> WebSocket(Proxy) 方向
        let t2w = async move {
            let mut buf = [0u8; 8192];
            loop {
                match tcp_read.read(&mut buf).await {
                    Ok(0) => {
                        info!("ローカル側の TCP 送信が終了しました(EOF)。");
                        break;
                    }, 
                    Ok(n) => {
                        // データを Message パケット(Command::Data)としてカプセル化
                        let packet = Message::new(Command::Data, buf[..n].to_vec());
                        if let Ok(bin) = packet.to_vec() {
                            // MessagePack 化したバイナリを WS で送信
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
            // 切断通知パケットを送ってから WS を閉じる
            if let Ok(msg) = Message::new(Command::Disconnect, vec![]).to_vec() {
                let _ = ws_write.send(WsMessage::Binary(msg)).await;
            }
            let _ = ws_write.close().await;
            info!("Local -> WS 転送終了。");
        };

        // [タスク B] WebSocket(Proxy) -> TCP(Local) 方向
        let w2t = async move {
            while let Some(msg) = ws_read.next().await {
                match msg {
                    Ok(WsMessage::Binary(bin)) => {
                        if let Ok(packet) = Message::from_slice(&bin) {
                            match packet.command {
                                // サーバー(ターゲット)から来た実データを、ローカル TCP へ書き出す
                                Command::Data => {
                                    if let Err(e) = tcp_write.write_all(&packet.payload).await {
                                        error!("TCP 書き込み失敗: {}", e);
                                        break;
                                    }
                                }
                                // サーバー側で TCP が切れた際の通知
                                Command::Disconnect => {
                                    info!("サーバーから切断の通知を受信しました。");
                                    break;
                                }
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
            info!("WS -> Local 転送終了。");
        };

        // select! により、どちらかの方向が終了した時点で、もう片方も終了に導きます。
        tokio::select! {
            _ = t2w => {},
            _ = w2t => {},
        }

        info!("トンネルセッションが正常にクローズされました。");
        Ok(())
    }
}
