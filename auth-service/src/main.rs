mod config;
mod db;
mod models;

use actix_web::{web, App, HttpServer, Responder};
use log::info;
use config::Config;
use db::create_pool;

async fn health_check() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    
    info!("Starting auth-service on port {}", config.port);

    // Initialize database connection pool
    let _pool = create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

