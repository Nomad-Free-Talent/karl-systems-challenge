mod config;
mod db;
mod models;
mod services;
mod handlers;

use actix_web::{web, App, HttpServer, Responder};
use log::info;
use config::Config;
use db::create_pool;
use sqlx::PgPool;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    config: Config,
}

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
    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    let app_state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(handlers::auth::register))
            )
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

