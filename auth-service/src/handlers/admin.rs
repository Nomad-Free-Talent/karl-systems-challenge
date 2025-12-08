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
    if User::find_by_username(&pool, &req.username).await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .is_some()
    {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    if User::find_by_email(&pool, &req.email).await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
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

// Role management endpoints

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::models::Role> for RoleResponse {
    fn from(role: crate::models::Role) -> Self {
        Self {
            id: role.id,
            name: role.name,
            description: role.description,
            created_at: role.created_at,
        }
    }
}

pub async fn list_roles(
    pool: web::Data<PgPool>,
) -> AppResult<impl Responder> {
    let roles = crate::models::Role::list(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list roles: {}", e)))?;

    let response: Vec<RoleResponse> = roles.into_iter().map(|r| r.into()).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_role(
    pool: web::Data<PgPool>,
    req: web::Json<CreateRoleRequest>,
) -> AppResult<impl Responder> {
    if req.name.is_empty() {
        return Err(AppError::BadRequest("Role name is required".to_string()));
    }

    let role = crate::models::Role::create(
        &pool,
        &req.name,
        req.description.as_deref(),
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            AppError::Conflict(format!("Role '{}' already exists", req.name))
        } else {
            AppError::Internal(format!("Failed to create role: {}", e))
        }
    })?;

    let response: RoleResponse = role.into();
    Ok(HttpResponse::Created().json(ApiResponse::new(response)))
}

// Permission management endpoints

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub id: Uuid,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::models::Permission> for PermissionResponse {
    fn from(permission: crate::models::Permission) -> Self {
        Self {
            id: permission.id,
            name: permission.name,
            resource: permission.resource,
            action: permission.action,
            created_at: permission.created_at,
        }
    }
}

pub async fn list_permissions(
    pool: web::Data<PgPool>,
) -> AppResult<impl Responder> {
    let permissions = crate::models::Permission::list(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list permissions: {}", e)))?;

    let response: Vec<PermissionResponse> = permissions.into_iter().map(|p| p.into()).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub name: String,
    pub resource: String,
    pub action: String,
}

pub async fn create_permission(
    pool: web::Data<PgPool>,
    req: web::Json<CreatePermissionRequest>,
) -> AppResult<impl Responder> {
    if req.name.is_empty() || req.resource.is_empty() || req.action.is_empty() {
        return Err(AppError::BadRequest("Permission name, resource, and action are required".to_string()));
    }

    let permission = crate::models::Permission::create(
        &pool,
        &req.name,
        &req.resource,
        &req.action,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            AppError::Conflict(format!("Permission '{}' already exists", req.name))
        } else {
            AppError::Internal(format!("Failed to create permission: {}", e))
        }
    })?;

    let response: PermissionResponse = permission.into();
    Ok(HttpResponse::Created().json(ApiResponse::new(response)))
}

// User-Role assignment endpoints

#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

pub async fn assign_role_to_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<AssignRoleRequest>,
) -> AppResult<impl Responder> {
    let user_id = path.into_inner();
    let role_id = req.role_id;

    // Verify user exists
    User::find_by_id(&pool, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

    // Verify role exists
    crate::models::Role::find_by_id(&pool, role_id)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Role with id {} not found", role_id)))?;

    // Assign role
    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        user_id,
        role_id
    )
    .execute(&**pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to assign role: {}", e)))?;

    Ok(HttpResponse::Created().json(ApiResponse::with_message(
        (),
        format!("Role assigned to user successfully")
    )))
}

pub async fn remove_role_from_user(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
) -> AppResult<impl Responder> {
    let (user_id, role_id) = path.into_inner();

    let result = sqlx::query!(
        "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2",
        user_id,
        role_id
    )
    .execute(&**pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to remove role: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User-role assignment not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}

// Role-Permission assignment endpoints

#[derive(Debug, Deserialize)]
pub struct AssignPermissionRequest {
    pub permission_id: Uuid,
}

pub async fn assign_permission_to_role(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<AssignPermissionRequest>,
) -> AppResult<impl Responder> {
    let role_id = path.into_inner();
    let permission_id = req.permission_id;

    // Verify role exists
    crate::models::Role::find_by_id(&pool, role_id)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Role with id {} not found", role_id)))?;

    // Verify permission exists
    crate::models::Permission::find_by_id(&pool, permission_id)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Permission with id {} not found", permission_id)))?;

    // Assign permission
    crate::models::Role::assign_permission(&pool, role_id, permission_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to assign permission: {}", e)))?;

    Ok(HttpResponse::Created().json(ApiResponse::with_message(
        (),
        format!("Permission assigned to role successfully")
    )))
}

