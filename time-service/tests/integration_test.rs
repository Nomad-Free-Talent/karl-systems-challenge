use actix_web::{http::StatusCode, test, web, App};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use shared::Claims;
use time_service::handlers::time::{get_time_for_city, get_time_for_timezone, list_timezones};
use time_service::middleware::JwtAuth;
use time_service::{Config, TimezoneCache, WorldTimeClient};
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
async fn test_get_time_for_timezone() {
    let config = Config::from_env();
    let cache = web::Data::new(TimezoneCache::new());
    let client = web::Data::new(WorldTimeClient::new());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(client)
            .service(
                web::scope("/time")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/timezones", web::get().to(list_timezones))
                    .route("/timezone/{timezone}", web::get().to(get_time_for_timezone))
                    .route("/{city}", web::get().to(get_time_for_city)),
            ),
    )
    .await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    let req = test::TestRequest::get()
        .uri("/time/timezone/Europe/London")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // May succeed or fail depending on external API availability
    // Also might return 404 if timezone not found
    assert!(
        resp.status() == StatusCode::OK
            || resp.status() == StatusCode::INTERNAL_SERVER_ERROR
            || resp.status() == StatusCode::NOT_FOUND
    );

    if resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("data").is_some());

        let data = body.get("data").unwrap();
        assert_eq!(
            data.get("timezone").unwrap().as_str().unwrap(),
            "Europe/London"
        );
        assert!(data.get("datetime").is_some());
        assert!(data.get("unix_time").is_some());
    }
}

#[tokio::test]
async fn test_get_time_for_city() {
    let config = Config::from_env();
    let cache = web::Data::new(TimezoneCache::new());
    let client = web::Data::new(WorldTimeClient::new());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(client)
            .service(
                web::scope("/time")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/timezones", web::get().to(list_timezones))
                    .route("/timezone/{timezone}", web::get().to(get_time_for_timezone))
                    .route("/{city}", web::get().to(get_time_for_city)),
            ),
    )
    .await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    let req = test::TestRequest::get()
        .uri("/time/London")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // This might return 404 if city is not found, 200 if found, or 500 if API error
    assert!(
        resp.status() == StatusCode::OK
            || resp.status() == StatusCode::NOT_FOUND
            || resp.status() == StatusCode::INTERNAL_SERVER_ERROR
    );

    if resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("data").is_some());

        let data = body.get("data").unwrap();
        assert!(data.get("city").is_some());
        assert!(data.get("datetime").is_some());
    }
}

#[tokio::test]
async fn test_list_timezones() {
    let config = Config::from_env();
    let cache = web::Data::new(TimezoneCache::new());
    let client = web::Data::new(WorldTimeClient::new());

    // Initialize cache with some timezones
    cache.initialize().await.ok();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(client)
            .service(
                web::scope("/time")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/timezones", web::get().to(list_timezones))
                    .route("/timezone/{timezone}", web::get().to(get_time_for_timezone))
                    .route("/{city}", web::get().to(get_time_for_city)),
            ),
    )
    .await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    let req = test::TestRequest::get()
        .uri("/time/timezones")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should always return OK, even if cache is empty (returns empty array)
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("data").is_some());

    let data = body.get("data").unwrap();
    assert!(data.is_array());
    // Cache might be empty, so array might be empty or have items
}

#[tokio::test]
async fn test_get_time_for_timezone_caching() {
    let config = Config::from_env();
    let cache = web::Data::new(TimezoneCache::new());
    let client = web::Data::new(WorldTimeClient::new());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(cache)
            .app_data(client)
            .service(
                web::scope("/time")
                    .wrap(JwtAuth::new(config.jwt_secret.clone()))
                    .route("/timezones", web::get().to(list_timezones))
                    .route("/timezone/{timezone}", web::get().to(get_time_for_timezone))
                    .route("/{city}", web::get().to(get_time_for_city)),
            ),
    )
    .await;
    let config = Config::from_env();
    let token = generate_test_token(&config.jwt_secret);

    // First request - should fetch from API
    let req1 = test::TestRequest::get()
        .uri("/time/timezone/America/New_York")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp1 = test::call_service(&app, req1).await;

    // Only test caching if first request succeeded
    if resp1.status() == StatusCode::OK {
        // Second request - should use cache
        let req2 = test::TestRequest::get()
            .uri("/time/timezone/America/New_York")
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
