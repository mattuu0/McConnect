use actix::prelude::*;
use actix_web_actors::ws;
use log::info;

/// 接続された個々の WebSocket セッションを管理するアクター。
/// 
/// Actix の Actor フレームワークを利用し、非同期にメッセージを処理します。
pub struct WsProxySession;

impl WsProxySession {
    pub fn new() -> Self {
        Self
    }
}

impl Actor for WsProxySession {
    type Context = ws::WebsocketContext<Self>;

    /// セッションが開始された時に呼ばれる
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket セッションが開始されました");
    }

    /// セッションが終了した時に呼ばれる
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket セッションが停止しました");
    }
}

/// WebSocket ストリーム経由で受信したメッセージのハンドリング
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsProxySession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            // Ping を受信した場合、自動的に Pong を返す
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            
            // テキストメッセージを受信した場合 (デバッグ用)
            Ok(ws::Message::Text(text)) => {
                info!("テキストメッセージを受信: {}", text);
                ctx.text(text); // エコーバック
            }
            
            // バイナリメッセージを受信した場合 (マイクラのパケットがここを通る)
            Ok(ws::Message::Binary(bin)) => {
                // TODO: 実際の接続先 TCP サーバーへデータを転送するロジックをここに実装
                info!("バイナリデータを受信: {} バイト", bin.len());
                ctx.binary(bin); // 現在はテストのためそのまま返す
            }
            
            // 切断要求を受信した場合
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            
            // その他のエラーや制御メッセージ
            Err(e) => {
                info!("WebSocket エラー: {:?}", e);
                ctx.stop();
            }
            _ => (),
        }
    }
}
