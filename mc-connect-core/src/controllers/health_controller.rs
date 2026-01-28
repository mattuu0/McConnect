use actix_web::{get, HttpResponse, Responder};

/// サーバーの死活監視用エンドポイント
/// 
/// `GET /health` にリクエストを送ることで、サーバーが正常に動作しているかを確認できます。
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
