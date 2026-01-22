use actix_web::{App, HttpServer, Responder, get, middleware::Logger, web};

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| App::new().service(index).service(hello).wrap(Logger::default()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
