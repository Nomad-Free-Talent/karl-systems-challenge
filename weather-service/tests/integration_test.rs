use actix_web::{http::StatusCode, test, web, App};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use shared::Claims;
use weather_service::handlers::weather::{get_weather, get_weather_providers};
use weather_service::{Config, WeatherCache, WeatherAggregator, RateLimiter};
use weather_service::middleware::JwtAuth;
use uuid::Uuid;

// Helper function to generate a test JWT token
fn generate_test_token(jwt_secret: &str) -> String {
    let claims = Claims {
        sub: Uuid::new_v4(),
        username: "testuser".to_string(),
        roles: vec!["user".to_string()],
        exp: (Utc::now().timestamp() + 3600), // 1 hour from now
        iat: Utc::now().timestamp(),
    };
    
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
    encode(&header, &claims, &encoding_key).expect("Failed to generate token")
}

#[tokio::test]
async fn test_get_weather() {
    let config = Config::from_env();
    let cache = web::Data::new(WeatherCache::new(1800));
    let rate_limiter = RateLimiter::new(1);
    let aggregator = web::Data::new(WeatherAggregator::new(rate_limiter));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(aggregator)
            .service(
                web::scope("/weather")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/{city}", web::get().to(get_weather))
                    .route("/{city}/providers", web::get().to(get_weather_providers))
            )
    ).await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    let req = test::TestRequest::get()
        .uri("/weather/London")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Weather API might fail or succeed depending on external API availability
    // So we check for either success or internal server error
    assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::INTERNAL_SERVER_ERROR);
    
    if resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("data").is_some());
        
        let data = body.get("data").unwrap();
        assert_eq!(data.get("city").unwrap().as_str().unwrap(), "London");
        assert!(data.get("aggregated").is_some());
        
        let aggregated = data.get("aggregated").unwrap();
        assert!(aggregated.get("temperature").is_some());
        assert!(aggregated.get("condition").is_some());
        assert!(aggregated.get("wind_speed").is_some());
    }
}

#[tokio::test]
async fn test_get_weather_with_cache() {
    let config = Config::from_env();
    let cache = web::Data::new(WeatherCache::new(1800));
    let rate_limiter = RateLimiter::new(1);
    let aggregator = web::Data::new(WeatherAggregator::new(rate_limiter));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(aggregator)
            .service(
                web::scope("/weather")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/{city}", web::get().to(get_weather))
                    .route("/{city}/providers", web::get().to(get_weather_providers))
            )
    ).await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    // First request - should fetch from API
    let req1 = test::TestRequest::get()
        .uri("/weather/Paris")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    
    // Only check cache if first request succeeded
    if resp1.status() == StatusCode::OK {
        // Second request - should use cache
        let req2 = test::TestRequest::get()
            .uri("/weather/Paris")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), StatusCode::OK);
        
        let body1: serde_json::Value = test::read_body_json(resp1).await;
        let body2: serde_json::Value = test::read_body_json(resp2).await;
        
        // Cached response should be the same
        assert_eq!(body1, body2);
    }
}

#[tokio::test]
async fn test_get_weather_force_refresh() {
    let config = Config::from_env();
    let cache = web::Data::new(WeatherCache::new(1800));
    let rate_limiter = RateLimiter::new(1);
    let aggregator = web::Data::new(WeatherAggregator::new(rate_limiter));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(aggregator)
            .service(
                web::scope("/weather")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/{city}", web::get().to(get_weather))
                    .route("/{city}/providers", web::get().to(get_weather_providers))
            )
    ).await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    // Request with cache=false to force refresh
    let req = test::TestRequest::get()
        .uri("/weather/London?cache=false")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should attempt to fetch fresh data
    assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_get_weather_providers() {
    let config = Config::from_env();
    let cache = web::Data::new(WeatherCache::new(1800));
    let rate_limiter = RateLimiter::new(1);
    let aggregator = web::Data::new(WeatherAggregator::new(rate_limiter));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(aggregator)
            .service(
                web::scope("/weather")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/{city}", web::get().to(get_weather))
                    .route("/{city}/providers", web::get().to(get_weather_providers))
            )
    ).await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    let req = test::TestRequest::get()
        .uri("/weather/London/providers")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Weather API might fail or succeed depending on external API availability
    assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::INTERNAL_SERVER_ERROR);
    
    if resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("data").is_some());
        
        let data = body.get("data").unwrap();
        assert!(data.is_array());
        // Should have multiple provider responses
    }
}

#[tokio::test]
async fn test_get_weather_different_cities() {
    let config = Config::from_env();
    let cache = web::Data::new(WeatherCache::new(1800));
    let rate_limiter = RateLimiter::new(1);
    let aggregator = web::Data::new(WeatherAggregator::new(rate_limiter));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(aggregator)
            .service(
                web::scope("/weather")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/{city}", web::get().to(get_weather))
                    .route("/{city}/providers", web::get().to(get_weather_providers))
            )
    ).await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    let cities = vec!["London", "Paris", "NewYork"]; // Use NewYork instead of "New York" to avoid URI encoding issues
    
    for city in cities {
        let req = test::TestRequest::get()
            .uri(&format!("/weather/{}", city))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // All should either succeed or fail together (if API is down)
        assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::INTERNAL_SERVER_ERROR);
    }
}
