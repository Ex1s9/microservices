use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod models {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum UserRole {
        Player,
        Developer,
        Admin,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: Uuid,
        pub email: String,
        pub username: String,
        pub created_at: DateTime<Utc>,
        pub role: UserRole,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateUserRequest {
        pub email: String,
        pub username: String,
        pub role: UserRole,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateUserRequest {
        pub email: Option<String>,
        pub username: Option<String>,
        pub role: Option<UserRole>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Game {
        pub id: Uuid,
        pub title: String,
        pub description: String,
        pub developer_id: Uuid,
        pub price: f64,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub status: GameStatus,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateGameRequest {
        pub title: String,
        pub description: String,
        pub developer_id: Uuid,
        pub price: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateGameRequest {
        pub title: Option<String>,
        pub description: Option<String>,
        pub developer_id: Option<Uuid>,
        pub price: Option<f64>,
        pub status: Option<GameStatus>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum GameStatus {
        Draft,
        Published,
        Archived
    }
}

pub mod utils {
    use super::*;

    pub fn create_user_from_request(request: CreateUserRequest) -> User {
        User {
            id: Uuid::new_v4(),
            email: request.email,
            username: request.username,
            created_at: Utc::now(),
            role: request.role,
        }
    }
}

pub mod errors {
    use std::fmt;

    #[derive(Debug)]
    pub enum ServiceError {
        NotFound(String),
        BadRequest(String),
        InternalError(String),
        Unauthorized,
    }

    impl fmt::Display for ServiceError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
                ServiceError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
                ServiceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
                ServiceError::Unauthorized => write!(f, "Unauthorized"),
            }
        }
    }

    impl std::error::Error for ServiceError {}
}

pub use models::*;
pub use utils::*;
pub use errors::*;