mod cache;
mod config;
mod handlers;
mod middleware;
mod services;

use actix_web::{web, App, HttpServer, Responder};
use cache::WeatherCache;
use config::Config;
use log::info;
use services::{RateLimiter, WeatherAggregator};

async fn health_check() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();

    info!("Starting weather-service on port {}", config.port);

    // Initialize cache (30 minute TTL)
    let cache = web::Data::new(WeatherCache::new(1800));

    // Initialize rate limiter (1 second minimum delay)
    let rate_limiter = RateLimiter::new(1);

    // Initialize aggregator
    let aggregator = web::Data::new(WeatherAggregator::new(rate_limiter));

    let jwt_secret = config.jwt_secret.clone();
    let port = config.port;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache.clone())
            .app_data(aggregator.clone())
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/weather")
                    .wrap(middleware::JwtAuth::new(jwt_secret.clone()))
                    .wrap(middleware::PermissionCheck::new("weather:read".to_string()))
                    .route("/{city}", web::get().to(handlers::weather::get_weather))
                    .route(
                        "/{city}/providers",
                        web::get().to(handlers::weather::get_weather_providers),
                    ),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
