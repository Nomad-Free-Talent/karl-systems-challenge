use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use shared::{ApiResponse, AppError, AppResult};
use crate::models::User;
use crate::services::hash_password;
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

