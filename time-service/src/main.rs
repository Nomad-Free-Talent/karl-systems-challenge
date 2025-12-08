mod config;
mod middleware;
mod cache;

use actix_web::{web, App, HttpServer, Responder};
use log::info;
use config::Config;
use cache::TimezoneCache;

async fn health_check() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    
    info!("Starting time-service on port {}", config.port);

    // Initialize and populate timezone cache
    let cache = web::Data::new(TimezoneCache::new());
    info!("Populating timezone cache...");
    if let Err(e) = cache.initialize().await {
        log::warn!("Failed to populate cache on startup: {}", e);
    } else {
        info!("Timezone cache populated successfully");
    }

    // Start background task to refresh cache periodically
    let cache_refresh = cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Refresh every hour
        loop {
            interval.tick().await;
            let timezones = cache_refresh.list_timezones().await;
            for tz in timezones {
                if let Err(e) = cache_refresh.refresh(&tz).await {
                    log::warn!("Failed to refresh timezone {}: {}", tz, e);
                }
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache.clone())
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}

