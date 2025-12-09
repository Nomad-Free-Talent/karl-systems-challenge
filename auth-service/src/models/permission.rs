use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRole {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermission {
    pub name: String,
    pub resource: String,
    pub action: String,
}

impl Role {
    pub async fn create(
        pool: &sqlx::PgPool,
        name: &str,
        description: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let role = sqlx::query_as!(
            Role,
            r#"
            INSERT INTO roles (name, description)
            VALUES ($1, $2)
            RETURNING id, name, description, created_at
            "#,
            name,
            description
        )
        .fetch_one(pool)
        .await?;

        Ok(role)
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let role = sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, description, created_at
            FROM roles
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    pub async fn find_by_name(
        pool: &sqlx::PgPool,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let role = sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, description, created_at
            FROM roles
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let roles = sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, description, created_at
            FROM roles
            ORDER BY name
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }

    pub async fn get_user_roles(
        pool: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let roles = sqlx::query_as!(
            Role,
            r#"
            SELECT r.id, r.name, r.description, r.created_at
            FROM roles r
            INNER JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            ORDER BY r.name
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }

    pub async fn get_permissions(
        pool: &sqlx::PgPool,
        role_id: Uuid,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        let permissions = sqlx::query_as!(
            Permission,
            r#"
            SELECT p.id, p.name, p.resource, p.action, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.name
            "#,
            role_id
        )
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }

    pub async fn assign_permission(
        pool: &sqlx::PgPool,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            VALUES ($1, $2)
            ON CONFLICT (role_id, permission_id) DO NOTHING
            "#,
            role_id,
            permission_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove_permission(
        pool: &sqlx::PgPool,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM role_permissions
            WHERE role_id = $1 AND permission_id = $2
            "#,
            role_id,
            permission_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

impl Permission {
    pub async fn create(
        pool: &sqlx::PgPool,
        name: &str,
        resource: &str,
        action: &str,
    ) -> Result<Self, sqlx::Error> {
        let permission = sqlx::query_as!(
            Permission,
            r#"
            INSERT INTO permissions (name, resource, action)
            VALUES ($1, $2, $3)
            RETURNING id, name, resource, action, created_at
            "#,
            name,
            resource,
            action
        )
        .fetch_one(pool)
        .await?;

        Ok(permission)
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let permission = sqlx::query_as!(
            Permission,
            r#"
            SELECT id, name, resource, action, created_at
            FROM permissions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(permission)
    }

    pub async fn find_by_name(
        pool: &sqlx::PgPool,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let permission = sqlx::query_as!(
            Permission,
            r#"
            SELECT id, name, resource, action, created_at
            FROM permissions
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(pool)
        .await?;

        Ok(permission)
    }

    pub async fn list(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let permissions = sqlx::query_as!(
            Permission,
            r#"
            SELECT id, name, resource, action, created_at
            FROM permissions
            ORDER BY resource, action
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }
}
