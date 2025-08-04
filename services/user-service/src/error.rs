#[derive(Debug)]
pub enum UserServiceError {
    Database(sqlx::Error),
    InvalidUuid(uuid::Error),
    PasswordHash(argon2::password_hash::Error),
    UserNotFound,
    ValidationError(String),
}

impl std::fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserServiceError::Database(e) => write!(f, "Database error: {}", e),
            UserServiceError::InvalidUuid(e) => write!(f, "Invalid UUID: {}", e),
            UserServiceError::PasswordHash(e) => write!(f, "Password hashing error: {}", e),
            UserServiceError::UserNotFound => write!(f, "User not found"),
            UserServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for UserServiceError {}

impl From<sqlx::Error> for UserServiceError {
    fn from(err: sqlx::Error) -> Self {
        UserServiceError::Database(err)
    }
}

impl From<uuid::Error> for UserServiceError {
    fn from(err: uuid::Error) -> Self {
        UserServiceError::InvalidUuid(err)
    }
}

impl From<argon2::password_hash::Error> for UserServiceError {
    fn from(err: argon2::password_hash::Error) -> Self {
        UserServiceError::PasswordHash(err)
    }
}
