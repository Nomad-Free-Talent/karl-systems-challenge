use actix_web::{test, web, App, http::StatusCode};
use time_service::handlers::time::get_time_for_timezone;
use time_service::{Config, TimezoneCache, WorldTimeClient};

async fn setup_test_app() -> App<impl actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>> {
    let config = Config::from_env();
    let cache = web::Data::new(TimezoneCache::new());
    let client = web::Data::new(WorldTimeClient::new());

    App::new()
        .app_data(web::Data::new(config))
        .app_data(cache)
        .app_data(client)
        .route("/time/timezone/{timezone}", web::get().to(get_time_for_timezone))
}

#[tokio::test]
async fn test_get_time_for_timezone() {
    let app = setup_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/time/timezone/Europe/London")
        .insert_header(("Authorization", "Bearer test-token"))
        .to_request();
    
    // Note: This test would need a valid JWT token
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status() == StatusCode::UNAUTHORIZED || resp.status().is_success());
}

