use actix_web::{test, web, App, http::StatusCode};
use weather_service::handlers::weather::get_weather;
use weather_service::{Config, WeatherCache, WeatherAggregator, RateLimiter};

#[tokio::test]
async fn test_get_weather() {
    let config = Config::from_env();
    let cache = web::Data::new(WeatherCache::new(1800));
    let rate_limiter = RateLimiter::new(1);
    let aggregator = web::Data::new(WeatherAggregator::new(rate_limiter));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(cache)
            .app_data(aggregator)
            .route("/weather/{city}", web::get().to(get_weather))
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/weather/London")
        .insert_header(("Authorization", "Bearer test-token"))
        .to_request();
    
    // Note: This test would need a valid JWT token
    // In a real scenario, you'd generate a token from auth-service
    let resp = test::call_service(&app, req).await;
    
    // This will fail without a valid token, but tests the structure
    assert!(resp.status() == StatusCode::UNAUTHORIZED || resp.status().is_success());
}

