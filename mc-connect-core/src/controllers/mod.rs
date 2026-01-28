pub mod health_controller;
pub mod ws_controller;

use actix_web::{web, App, HttpServer};
use log::info;

use crate::models::packet::AllowedPort;

/// サーバーを起動するためのメインエントリーポイント
/// 
/// # 引数
/// * `host` - バインドするホスト名 (例: "127.0.0.1")
/// * `port` - 待受ポート番号
/// * `allowed_ports` - 許可するターゲットポートのリスト
pub async fn start_server(host: &str, port: u16, allowed_ports: Vec<AllowedPort>) -> std::io::Result<()> {
    info!("McConnect サーバーを起動中: {}:{}", host, port);
    info!("許可されたポート: {:?}", allowed_ports);

    let allowed_ports = web::Data::new(allowed_ports);

    HttpServer::new(move || {
        App::new()
            .app_data(allowed_ports.clone())
            // ヘルスチェックエンドポイントの登録
            .service(health_controller::health_check)
            // WebSocket プロキシエンドポイントの登録
            .route("/ws", web::get().to(ws_controller::ws_proxy))
    })
    .bind((host, port))?
    .run()
    .await
}
