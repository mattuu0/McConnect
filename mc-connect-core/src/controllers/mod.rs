pub mod health_controller;
pub mod ws_controller;

use actix_web::{web, App, HttpServer};
use log::info;

/// サーバーを起動するためのメインエントリーポイント
/// 
/// # 引数
/// * `host` - バインドするホスト名 (例: "127.0.0.1")
/// * `port` - 待受ポート番号
pub async fn start_server(host: &str, port: u16) -> std::io::Result<()> {
    info!("McConnect サーバーを起動中: {}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            // ヘルスチェックエンドポイントの登録
            .service(health_controller::health_check)
            // WebSocket プロキシエンドポイントの登録
            .route("/ws", web::get().to(ws_controller::ws_proxy))
    })
    .bind((host, port))?
    .run()
    .await
}
