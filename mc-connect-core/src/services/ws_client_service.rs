use futures_util::{StreamExt, SinkExt};
use log::{info, error};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

/// WebSocket クライアントとしての動作を管理するサービス
pub struct WsClientService;

impl WsClientService {
    /// 指定された URL の WebSocket プロキシサーバーに接続する
    /// 
    /// # 引数
    /// * `url_str` - 接続先 URL (例: "ws://example.com/ws")
    pub async fn connect(url_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = Url::parse(url_str)?;
        
        info!("WebSocket サーバーに接続中: {}", url);
        
        // サーバーへのハンドシェイクを実行
        let (ws_stream, response) = connect_async(url).await?;
        
        info!("接続成功。ステータスコード: {}", response.status());
        
        // ストリームを送信（write）と受信（read）に分離
        let (mut write, mut read) = ws_stream.split();
        
        // 受信用ループを別タスクで開始
        // TODO: TCP 側（ローカルのマイクラ接続）から来たデータを WebSocket に流す処理を統合する
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        info!("サーバーからテキストを受信: {}", text);
                    }
                    Ok(Message::Binary(bin)) => {
                        info!("サーバーからバイナリを受信: {} バイト", bin.len());
                        // TODO: 受信したデータをローカルの TCP ストリームに書き込む
                    }
                    Ok(Message::Ping(payload)) => {
                        // Ping に対する自動 Pong 応答
                        let _ = write.send(Message::Pong(payload)).await;
                    }
                    Ok(Message::Close(_)) => {
                        info!("サーバーが接続を閉じました");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket エラー: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
