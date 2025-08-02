use actix_web::{
    middleware::{self, Next},
    web, App, HttpServer, HttpResponse, Error,
    dev::{ServiceRequest, ServiceResponse},
    HttpMessage,
};
use serde_json;

use actix_cors::Cors;
use uuid::Uuid;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;


struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
        }
    }

    fn check_rate_limit(&self, ip: &str, limit: usize, window: Duration) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        let timestamps = requests.entry(ip.to_string()).or_insert_with(Vec::new);
        
        timestamps.retain(|&t| now.duration_since(t) < window);
        
        if timestamps.len() >= limit {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

pub mod user {
    tonic::include_proto!("user");
}

#[derive(Deserialize)]
struct CreateUserDto {
    email: String,
    username: String,
    password: String,
    role: String,
}

#[derive(Serialize)]
struct UserDto {
    id: String,
    email: String,
    username: String,
    role: String,
    created_at: String,
}

#[derive(Deserialize)]
struct UpdateUserDto {
    email: Option<String>,
    username: Option<String>,
    password: Option<String>,
    role: Option<String>,
}

#[derive(Deserialize)]
struct ListUsersQuery {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Serialize)]
struct ListUsersHttpResponse {
    users: Vec<UserDto>,
    total: i32,
}


struct AppState {
    user_client: user::user_service_client::UserServiceClient<Channel>,
}

async fn create_user(
    data: web::Data<AppState>,
    json: web::Json<CreateUserDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let role = match json.role.as_str() {
        "player" => 0,
        "developer" => 1,
        "admin" => 2,
        _ => return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid role"
        }))),
    };

    let request = tonic::Request::new(user::CreateUserRequest {
        email: json.email.clone(),
        username: json.username.clone(),
        password: json.password.clone(),
        role,
    });

    let mut client = data.user_client.clone();
    match client.create_user(request).await {
        Ok(response) => {
            let user = response.into_inner();
            
            let user_dto = UserDto {
                id: user.id,
                email: user.email,
                username: user.username,
                role: proto_role_to_string(user.role),
                created_at: user.created_at
                    .map(|ts| format!("{}", ts.seconds))
                    .unwrap_or_default(),
            };
            
            Ok(HttpResponse::Ok().json(user_dto))
        }
        Err(status) => {
            match status.code() {
                tonic::Code::InvalidArgument => {
                    Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": status.message()
                    })))
                }
                tonic::Code::AlreadyExists => {
                    Ok(HttpResponse::Conflict().json(serde_json::json!({
                        "error": "User with this email or username already exists"
                    })))
                }
                _ => {
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": status.message()
                    })))
                }
            }
        }
    }
}

async fn get_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();
    
    let request = tonic::Request::new(user::GetUserRequest {
        id: user_id,
    });
    
    let mut client = data.user_client.clone();
    match client.get_user(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            if let Some(user) = resp.user {
                let user_dto = UserDto {
                    id: user.id,
                    email: user.email,
                    username: user.username,
                    role: proto_role_to_string(user.role),
                    created_at: user.created_at
                        .map(|ts| format!("{}", ts.seconds))
                        .unwrap_or_default(),
                };
                Ok(HttpResponse::Ok().json(user_dto))
            } else {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                })))
            }
        }
        Err(status) => {
            match status.code() {
                tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                }))),
                _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": status.message()
                }))),
            }
        }
    }
}

async fn update_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
    json: web::Json<UpdateUserDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();
    
    if uuid::Uuid::parse_str(&user_id).is_err() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid user ID format"
        })));
    }
    
    let role = if let Some(role_str) = &json.role {
        match role_str.as_str() {
            "player" => Some(0),
            "developer" => Some(1),
            "admin" => Some(2),
            _ => return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid role. Must be: player, developer, or admin"
            }))),
        }
    } else {
        None
    };

    let request = tonic::Request::new(user::UpdateUserRequest {
        id: user_id,
        email: json.email.clone(),
        username: json.username.clone(),
        password: json.password.clone(),
        role,
    });

    let mut client = data.user_client.clone();
    match client.update_user(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            
            match resp.user {
                Some(user) => {
                    let user_dto = UserDto {
                        id: user.id,
                        email: user.email,
                        username: user.username,
                        role: proto_role_to_string(user.role),
                        created_at: user.created_at
                            .map(|ts| format!("{}", ts.seconds))
                            .unwrap_or_default(),
                    };
                    Ok(HttpResponse::Ok().json(user_dto))
                }
                None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Server returned empty response"
                })))
            }
        }
        Err(status) => {
            match status.code() {
                tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                }))),
                tonic::Code::InvalidArgument => Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": status.message()
                }))),
                tonic::Code::AlreadyExists => Ok(HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Email or username already taken"
                }))),
                _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Internal error: {}", status.message())
                })))
            }
        }
    }
}

async fn delete_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();

    let request = tonic::Request::new(user::DeleteUserRequest {
        id: user_id,
    });

    let mut client = data.user_client.clone();
    match client.delete_user(request).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "User deleted successfully"
            })))
        }
        Err(status) => {
            match status.code() {
                tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                }))),
                _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": status.message()
                }))),
            }
        }
    }
}

async fn users_list(
    data: web::Data<AppState>,
    query: web::Query<ListUsersQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let request = tonic::Request::new(user::ListUsersRequest {
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
        role: None,
    });

    let mut client = data.user_client.clone();
    match client.list_users(request).await {
        Ok(response) => {
            let resp = response.into_inner();

            let user_dtos: Vec<UserDto> = resp.users.into_iter().map(|user| UserDto {
                id: user.id,
                email: user.email,
                username: user.username,
                role: proto_role_to_string(user.role),
                created_at: user.created_at
                    .map(|ts| format!("{}", ts.seconds))
                    .unwrap_or_default(),
            }).collect();

            Ok(HttpResponse::Ok().json(ListUsersHttpResponse {
                users: user_dtos,
                total: resp.total,
            }))
        }
        Err(status) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            })))
        }
    }
}

fn proto_role_to_string(role: i32) -> String {
    match role {
        0 => "player".to_string(),
        1 => "developer".to_string(),
        2 => "admin".to_string(),
        _ => "unknown".to_string(),
    }
}

async fn rate_limit_middleware(
    req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody + 'static>,
) -> Result<ServiceResponse<actix_web::body::BoxBody>, Error> {
    let rate_limiter = req.app_data::<web::Data<RateLimiter>>().unwrap();
    let ip = req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    if !rate_limiter.check_rate_limit(&ip, 100, Duration::from_secs(60)) {
        return Ok(req.into_response(
            HttpResponse::TooManyRequests()
                .json(serde_json::json!({
                    "error": "Rate limit exceeded. Please try again later."
                }))
                .map_into_boxed_body()
        ));
    }
    
    let res = next.call(req).await?;
    Ok(res.map_into_boxed_body())
}

async fn request_id_middleware(
    req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody + 'static>,
) -> Result<ServiceResponse<actix_web::body::BoxBody>, Error> {
    let request_id = Uuid::new_v4().to_string();
    
    req.extensions_mut().insert(request_id.clone());
    
    println!("Request ID: {} - {} {}", request_id, req.method(), req.path());
    
    let mut res = next.call(req).await?;
    
    res.headers_mut().insert(
        actix_web::http::header::HeaderName::from_static("x-request-id"),
        actix_web::http::header::HeaderValue::from_str(&request_id).unwrap(),
    );
    
    Ok(res.map_into_boxed_body())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let user_client = user::user_service_client::UserServiceClient::connect("http://[::1]:50051")
        .await
        .expect("Failed to connect to user service");
    
    let app_state = web::Data::new(AppState {
        user_client,
    });

    let rate_limiter = web::Data::new(RateLimiter::new());
    
    println!("Gateway service listening on http://localhost:8080");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // React
            .allowed_origin("http://localhost:5173") // Vite
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .expose_headers(vec!["x-request-id"])
            .max_age(3600);

            App::new()
            .app_data(app_state.clone())
            .app_data(rate_limiter.clone())
            .wrap(middleware::from_fn(request_id_middleware))
            .wrap(middleware::from_fn(rate_limit_middleware))
            .wrap(cors)
            .wrap(middleware::Logger::new("%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
            .route("/api/users", web::post().to(create_user))
            .route("/api/users/{id}", web::get().to(get_user))
            .route("/api/users/{id}", web::put().to(update_user))
            .route("/api/users/{id}", web::delete().to(delete_user))
            .route("/api/users", web::get().to(users_list))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}