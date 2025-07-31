use tonic::transport::Server;
use tonic::{Request, Response, Status};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use dotenv::dotenv;
use std::env;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;

use uuid::Uuid;

use error::UserServiceError;

pub mod user {
    tonic::include_proto!("user");
}

mod db;
mod error;
mod validation;

pub struct UserServiceImpl {
    pool: PgPool,
}

impl UserServiceImpl {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl user::user_service_server::UserService for UserServiceImpl {
    async fn get_user(
        &self,
        request: Request<user::GetUserRequest>,
    ) -> Result<Response<user::GetUserResponse>, Status> {
        let user_id = request.into_inner().id;

        let user_record = db::get_user_by_id(&self.pool, &user_id)
            .await
            .map_err(user_service_error_to_status)?;

        let user_msg = user::UserMessage {
            id: user_record.id.to_string(),
            email: user_record.email,
            username: user_record.username,
            role: db_role_to_proto(user_record.role),
            created_at: Some(datetime_to_timestamp(user_record.created_at)),
        };

        Ok(Response::new(user::GetUserResponse {
            user: Some(user_msg),
        }))
    }

    async fn create_user(
        &self,
        request: Request<user::CreateUserRequest>,
    ) -> Result<Response<user::UserMessage>, Status> {
        let req = request.into_inner();

        if let Err(e) = validation::validate_create_user_request(&req) {
            return Err(Status::invalid_argument(e));
        }

        let password_hash = db::hash_password(&req.password)
            .map_err(|e| Status::internal(format!("Password hash failed: {}", e)))?;

        let user_record = db::create_user(&self.pool, &req, &password_hash)
            .await
            .map_err(user_service_error_to_status)?;

        let user_msg = user::UserMessage {
            id: user_record.id.to_string(),
            email: user_record.email,
            username: user_record.username,
            role: db_role_to_proto(user_record.role),
            created_at: Some(datetime_to_timestamp(user_record.created_at)),
        };

        Ok(Response::new(user_msg))
    }

    async fn update_user(
        &self,
        request: Request<user::UpdateUserRequest>,
    ) -> Result<Response<user::UpdateUserResponse>, Status> {
        let req = request.into_inner();

        if let Err(e) = validation::validate_update_user_request(&req) {
            return Err(Status::invalid_argument(e));
        }

        let user_record = db::update_user(&self.pool, &req)
            .await
            .map_err(user_service_error_to_status)?;

        let user_msg = user::UserMessage {
            id: user_record.id.to_string(),
            email: user_record.email,
            username: user_record.username,
            role: db_role_to_proto(user_record.role),
            created_at: Some(datetime_to_timestamp(user_record.created_at)),
        };

        Ok(Response::new(user::UpdateUserResponse {
            user: Some(user_msg),
        }))
    }

    async fn delete_user(
        &self,
        request: Request<user::DeleteUserRequest>,
    ) -> Result<Response<user::DeleteUserResponse>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?;

        let success = db::delete_user(&self.pool, &id)
            .await
            .map_err(user_service_error_to_status)?;

        Ok(Response::new(user::DeleteUserResponse {
            success,
            message: "User deleted successfully".to_string(),
        }))
    }


    async fn list_users(
        &self,
        request: Request<user::ListUsersRequest>,
    ) -> Result<Response<user::ListUsersResponse>, Status> {
        let req = request.into_inner();
        
        let users = db::list_users(
            &self.pool, 
            Some(req.limit), 
            Some(req.offset)
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list users: {}", e)))?;
        
        let user_messages: Vec<user::UserMessage> = users
            .into_iter()
            .map(|user| user::UserMessage {
                id: user.id.to_string(),
                email: user.email,
                username: user.username,
                role: db_role_to_proto(user.role),
                created_at: Some(datetime_to_timestamp(user.created_at)),
            })
            .collect();
        
        let total = user_messages.len() as i32;
        
        Ok(Response::new(user::ListUsersResponse {
            users: user_messages,
            total,
        }))
    }
}

pub fn user_service_error_to_status(err: UserServiceError) -> Status {
    match err {
        UserServiceError::Database(sqlx_err) => {
            match sqlx_err {
                sqlx::Error::RowNotFound => Status::not_found("User not found"),
                _ => Status::internal(format!("Database error: {}", sqlx_err)),
            }
        },
        UserServiceError::InvalidUuid(_) => Status::invalid_argument("Invalid user ID format"),
        UserServiceError::PasswordHash(_) => Status::internal("Password processing failed"),
        UserServiceError::UserNotFound => Status::not_found("User not found"),
        UserServiceError::ValidationError(msg) => Status::invalid_argument(msg),
    }
}

pub fn datetime_to_timestamp(datetime: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: datetime.timestamp(),
        nanos: datetime.timestamp_subsec_nanos() as i32,
    }
}

fn db_role_to_proto(role: db::DbUserRole) -> i32 {
    match role {
        db::DbUserRole::Player => 0,
        db::DbUserRole::Developer => 1,
        db::DbUserRole::Admin => 2,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    let addr = "[::1]:50051".parse()?;
    let user_service = UserServiceImpl::new(pool);

    println!("UserService listening on {}", addr);

    Server::builder()
        .add_service(user::user_service_server::UserServiceServer::new(user_service))
        .serve(addr)
        .await?;

    Ok(())
}
