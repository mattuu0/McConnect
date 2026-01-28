use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use crate::services::proxy_service::WsProxySession;
use log::info;

use crate::models::packet::AllowedPort;

/// WebSocket 通信を開始するためのハンドラ
/// 
/// HTTP リクエストを WebSocket プロトコルにアップグレードし、
/// 以降の通信を WsProxySession アクターに委ねます。
pub async fn ws_proxy(
    req: HttpRequest, 
    stream: web::Payload,
    allowed_ports: web::Data<Vec<AllowedPort>>
) -> Result<HttpResponse, Error> {
    info!("WebSocket へのアップグレード要求を受信: {:?}", req.peer_addr());
    
    // Actix アクターを使用して WebSocket セッションを開始
    ws::start(WsProxySession::new(allowed_ports.get_ref().clone()), &req, stream)
}
