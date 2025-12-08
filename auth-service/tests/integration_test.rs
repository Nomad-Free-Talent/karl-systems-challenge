use actix_web::{test, web, App, http::StatusCode};
use auth_service::handlers::auth::{register, login, RegisterRequest, LoginRequest};
use auth_service::{Config, create_pool};
use sqlx::PgPool;
use std::env;

async fn setup_test_pool() -> PgPool {
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/auth_db_test".to_string());
    
    create_pool(&database_url)
        .await
        .expect("Failed to create test database pool")
}

#[tokio::test]
async fn test_register_user() {
    let pool = setup_test_pool().await;
    let config = Config::from_env();
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .route("/register", web::post().to(register))
    ).await;
    
    let register_req = RegisterRequest {
        username: format!("testuser_{}", uuid::Uuid::new_v4()),
        email: format!("test_{}@example.com", uuid::Uuid::new_v4()),
        password: "testpassword123".to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&register_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status() == StatusCode::CREATED);
}

#[tokio::test]
async fn test_login_user() {
    let pool = setup_test_pool().await;
    let config = Config::from_env();
    
    // First register a user
    let register_req = RegisterRequest {
        username: format!("loginuser_{}", uuid::Uuid::new_v4()),
        email: format!("login_{}@example.com", uuid::Uuid::new_v4()),
        password: "loginpassword123".to_string(),
    };
    
    let app_register = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .route("/register", web::post().to(register))
    ).await;
    
    let req_register = test::TestRequest::post()
        .uri("/register")
        .set_json(&register_req)
        .to_request();
    
    let _resp_register = test::call_service(&app_register, req_register).await;
    
    // Now try to login
    let login_req = LoginRequest {
        username: register_req.username.clone(),
        password: register_req.password.clone(),
    };
    
    let app_login = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .route("/login", web::post().to(login))
    ).await;
    
    let req_login = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_req)
        .to_request();
    
    let resp_login = test::call_service(&app_login, req_login).await;
    assert!(resp_login.status().is_success());
}
