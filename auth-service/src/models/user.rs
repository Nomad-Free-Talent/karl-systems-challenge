use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub is_active: Option<bool>,
}

impl User {
    pub async fn create(
        pool: &sqlx::PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at, updated_at, is_active
            "#,
            username,
            email,
            password_hash
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_username(
        pool: &sqlx::PgPool,
        username: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(
        pool: &sqlx::PgPool,
        email: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn update(
        pool: &sqlx::PgPool,
        id: Uuid,
        update: &UpdateUser,
    ) -> Result<Option<Self>, sqlx::Error> {
        let mut user = match Self::find_by_id(pool, id).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        if let Some(ref username) = update.username {
            user.username = username.clone();
        }
        if let Some(ref email) = update.email {
            user.email = email.clone();
        }
        if let Some(ref password_hash) = update.password {
            user.password_hash = password_hash.clone();
        }
        if let Some(is_active) = update.is_active {
            user.is_active = is_active;
        }

        let updated = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET username = $1, email = $2, password_hash = COALESCE($3, password_hash),
                is_active = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING id, username, email, password_hash, created_at, updated_at, is_active
            "#,
            user.username,
            user.email,
            update.password.as_ref(),
            user.is_active,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
