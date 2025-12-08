use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use shared::{ApiResponse, AppError, AppResult};
use crate::models::User;
use crate::services::{hash_password, verify_password, generate_token, Claims};
use crate::models::permission::Role;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: uuid::Uuid,
    pub username: String,
    pub roles: Vec<String>,
}

pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<crate::config::Config>,
    req: web::Json<RegisterRequest>,
) -> AppResult<impl Responder> {
    // Validate input
    if req.username.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("Username, email, and password are required".to_string()));
    }

    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters long".to_string()));
    }

    // Check if username or email already exists
    if User::find_by_username(&pool, &req.username).await?
        .is_some()
    {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    if User::find_by_email(&pool, &req.email).await?
        .is_some()
    {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&req.password)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    // Create user
    let user = User::create(
        &pool,
        &req.username,
        &req.email,
        &password_hash,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create user: {}", e)))?;

    // Assign default "user" role
    let user_role = Role::find_by_name(&pool, "user")
        .await
        .map_err(|e| AppError::Internal(format!("Failed to find user role: {}", e)))?
        .ok_or_else(|| AppError::Internal("Default user role not found".to_string()))?;

    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        user.id,
        user_role.id
    )
    .execute(&**pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to assign role: {}", e)))?;

    let response = RegisterResponse {
        user_id: user.id,
        username: user.username,
        email: user.email,
    };

    Ok(HttpResponse::Created().json(ApiResponse::new(response)))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<crate::config::Config>,
    req: web::Json<LoginRequest>,
) -> AppResult<impl Responder> {
    // Find user by username
    let user = User::find_by_username(&pool, &req.username)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    // Check if user is active
    if !user.is_active {
        return Err(AppError::Forbidden("User account is inactive".to_string()));
    }

    // Verify password
    verify_password(&req.password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("Password verification error: {}", e)))?
        .then_some(())
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    // Get user roles
    let roles = Role::get_user_roles(&pool, user.id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get user roles: {}", e)))?;

    let role_names: Vec<String> = roles.iter().map(|r| r.name.clone()).collect();

    // Generate JWT token
    let claims = Claims::new(user.id, user.username.clone(), role_names.clone());
    let token = generate_token(&claims, &config.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))?;

    let response = LoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        roles: role_names,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

