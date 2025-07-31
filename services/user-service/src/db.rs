use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use serde::{Deserialize, Serialize};
use crate::UserServiceError;


#[derive(Debug, sqlx::Type, Clone, Copy, Serialize, Deserialize)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum DbUserRole {
     Player,
     Developer,
     Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbUser {
     pub id: Uuid,
     pub email: String,
     pub username: String,
     pub created_at: DateTime<Utc>,
     pub role: DbUserRole,
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
     let salt = SaltString::generate(&mut OsRng);
     let argon2 = Argon2::default();
     Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
}

pub async fn get_user_by_id(pool: &PgPool, id: &str) -> Result<DbUser, UserServiceError> {
     let uuid = Uuid::parse_str(id)
          .map_err(|_| UserServiceError::UserNotFound)?; 

     let record = sqlx::query_as!(
          DbUser,
          r#"
          SELECT id, email, username, created_at, role as "role: DbUserRole"
          FROM users
          WHERE id = $1
          "#,
          uuid
     )
     .fetch_one(pool)
     .await?;

     Ok(DbUser {
          id: record.id,
          email: record.email,
          username: record.username,
          created_at: record.created_at,
          role: record.role,
     })
}

pub async fn create_user(
     pool: &PgPool,
     req: &crate::user::CreateUserRequest,
     password_hash: &str,
) -> Result<DbUser, UserServiceError> {
     let id = Uuid::new_v4();
     let now = Utc::now();
     
     let db_role = match req.role {
          0 => DbUserRole::Player,
          1 => DbUserRole::Developer,
          2 => DbUserRole::Admin,
          _ => DbUserRole::Player,
     };

     let record = sqlx::query_as!(
          DbUser,
          r#"
          INSERT INTO users (id, email, username, password_hash, role, created_at, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, $6)
          RETURNING id, email, username, created_at, role as "role: DbUserRole"
          "#,
          id,
          req.email,
          req.username,
          password_hash,
          db_role as DbUserRole,
          now
     )
     .fetch_one(pool)
     .await?;

     Ok(DbUser {
          id: record.id,
          email: record.email,
          username: record.username,
          created_at: record.created_at,
          role: record.role,
     })
}

pub async fn update_user(
     pool: &PgPool,
     req: &crate::user::UpdateUserRequest,
) -> Result<DbUser, UserServiceError> {
     let id = Uuid::parse_str(&req.id)?;

     let password_hash = if let Some(password) = &req.password {
          Some(hash_password(password)?)
     } else {
          None
     };

     let record = sqlx::query_as!(
          DbUser,
          r#"
          UPDATE users 
          SET 
               email = COALESCE($2, email),
               username = COALESCE($3, username),
               password_hash = COALESCE($4, password_hash),
               updated_at = NOW()
          WHERE id = $1
          RETURNING id, email, username, created_at, role as "role: DbUserRole"
          "#,
          id,
          req.email,
          req.username,
          password_hash
     )
     .fetch_one(pool)
     .await?;

     Ok(record)
}

pub async fn delete_user(pool: &PgPool, id: &Uuid) -> Result<bool, UserServiceError> {
     let result = sqlx::query!(
          "DELETE FROM users WHERE id = $1",
          id
     )
     .execute(pool)
     .await?;
     
     if result.rows_affected() > 0 {
          Ok(true)
     } else {
          Err(UserServiceError::UserNotFound)
     }
}

pub async fn list_users(
     pool: &PgPool,
     limit: Option<i32>,
     offset: Option<i32>
) -> Result<Vec<DbUser>, UserServiceError> {
     let limit = limit.unwrap_or(50);
     let offset = offset.unwrap_or(0);

     let records = sqlx::query_as!(
          DbUser,
          r#"
          SELECT id, email, username, created_at, role as "role: DbUserRole"
          FROM users
          ORDER BY created_at DESC
          LIMIT $1 OFFSET $2
          "#,
          limit as i64,
          offset as i64,
     )
     .fetch_all(pool)
     .await?;

     Ok(records)
}