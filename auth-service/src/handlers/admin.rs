use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use shared::{ApiResponse, AppError, AppResult};
use crate::models::User;
use crate::services::{hash_password, Claims};
use crate::handlers::auth::RegisterRequest;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

pub async fn list_users(
    pool: web::Data<PgPool>,
) -> AppResult<impl Responder> {
    let users = User::list(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list users: {}", e)))?;

    let response: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

pub async fn get_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> AppResult<impl Responder> {
    let user_id = path.into_inner();
    
    let user = User::find_by_id(&pool, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get user: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

    let response: UserResponse = user.into();
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

pub async fn create_user(
    pool: web::Data<PgPool>,
    req: web::Json<RegisterRequest>,
) -> AppResult<impl Responder> {
    // Validate input
    if req.username.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("Username, email, and password are required".to_string()));
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

    let response: UserResponse = user.into();
    Ok(HttpResponse::Created().json(ApiResponse::new(response)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn update_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserRequest>,
) -> AppResult<impl Responder> {
    let user_id = path.into_inner();

    let password_hash = if let Some(ref password) = req.password {
        Some(hash_password(password)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?)
    } else {
        None
    };

    let update = crate::models::user::UpdateUser {
        username: req.username.clone(),
        email: req.email.clone(),
        password: password_hash,
        is_active: req.is_active,
    };

    let user = User::update(&pool, user_id, &update)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update user: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

    let response: UserResponse = user.into();
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

pub async fn delete_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> AppResult<impl Responder> {
    let user_id = path.into_inner();

    let deleted = User::delete(&pool, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete user: {}", e)))?;

    if !deleted {
        return Err(AppError::NotFound(format!("User with id {} not found", user_id)));
    }

    Ok(HttpResponse::NoContent().finish())
}

