mod config;
mod middleware;

use actix_web::{web, App, HttpServer, Responder};
use log::info;
use config::Config;

async fn health_check() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    
    info!("Starting weather-service on port {}", config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

