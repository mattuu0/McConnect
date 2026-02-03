use actix::prelude::*;
use actix_web_actors::ws;
use tokio::sync::mpsc;
use crate::models::packet::{AllowedPort, Message, Command, ConnectResponsePayload};

/// [WsProxySession]
/// ゲートウェイ（サーバー）側で、WebSocket接続1つにつき、1つ生成されるアクターです。
/// 
/// このアクターは、クライアントからの WebSocket の流れと、
/// 背後にあるターゲット（Minecraftサーバー等）への TCP 接続の流れを橋渡しします。
/// Actix アクターフレームワークにより、イベント駆動で動作します。
pub struct WsProxySession {
    /// ターゲット TCP サーバー（背後のサーバー）へデータを送信するためのチャネル。
    /// 接続が確立されるまで、または切断された後は `None` になります。
    pub tcp_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,

    /// サーバー設定により許可されているポートとプロトコルのリスト。
    /// 接続要求 (`Connect`) が来た際に、このリストに合致するかチェックします。
    pub allowed_ports: Vec<AllowedPort>,

    /// トンネルの初期化（ターゲットへの接続確立）が完了しているかどうか。
    /// 一度初期化した後に再度 `Connect` が来た場合のガードとして使用します。
    pub initialized: bool,
}

impl WsProxySession {
    /// 許可ポート情報を保持した新しいセッションアクターを作成します。
    pub fn new(allowed_ports: Vec<AllowedPort>) -> Self {
        Self {
            tcp_tx: None,
            allowed_ports,
            initialized: false,
        }
    }

    /// [send_packet]
    /// コンテンツ（コマンドとデータ）を受け取り、MessagePack 形式でカプセル化して
    /// WebSocket クライアントへバイナリデータとして送信します。
    pub fn send_packet(&self, ctx: &mut ws::WebsocketContext<Self>, command: Command, payload: Vec<u8>) {
        let msg = Message::new(command, payload);
        match msg.to_vec() {
            Ok(bin) => ctx.binary(bin),
            Err(e) => log::error!("Packet serialization error: {}", e),
        }
    }

    /// [stop_with_error]
    /// 接続失敗などの致命的なエラーが発生した際に、
    /// クライアントへ失敗パケットを送信した上で、セッション（アクター）を終了します。
    pub fn stop_with_error(&self, ctx: &mut ws::WebsocketContext<Self>, message: String) {
        let res = ConnectResponsePayload { success: false, message: message.clone() };
        if let Ok(msg) = Message::from_payload(Command::ConnectResponse, &res) {
            if let Ok(bin) = msg.to_vec() {
                ctx.binary(bin);
            }
        }
        log::error!("Closing session due to error: {}", message);
        ctx.stop();
    }
}

/// Actor トレイトのの実装: セッションの開始と終了のフック。
impl Actor for WsProxySession {
    type Context = ws::WebsocketContext<Self>;

    /// アクター（接続）が開始された時に呼ばれます。
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("WebSocket session started. Waiting for Connect packet...");
    }

    /// アクターが停止する直前に呼ばれます。
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::info!("WebSocket session stopped. Cleaning up resources...");
        // 備考: tcp_tx がここでドロップされることで、TCP書き込みループの rx 側が閉じ、
        // 関連する tokio タスクも自動的に終了する仕組みになっています。
    }
}
