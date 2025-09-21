use actix_web::{
    App, Error, HttpMessage, HttpResponse, HttpServer,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{self, Next},
    web,
};
use serde_json;

use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tonic::transport::Channel;
use uuid::Uuid;

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

pub mod game {
    tonic::include_proto!("game");
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

// Game DTOs and handlers would go here similarly
#[derive(Deserialize)]
struct CreateGameDto {
    name: String,
    description: Option<String>,
    developer_id: String,
    publisher_id: Option<String>,
    cover_image: Option<String>,
    trailer_url: Option<String>,
    release_date: Option<String>,
    tags: Vec<String>,
    platforms: Vec<String>,
    screenshots: Vec<String>,
    price: f64,
    status: String,
    categories: Vec<String>,
}

#[derive(Serialize)]
struct GameDto {
    id: String,
    name: String,
    description: Option<String>,
    developer_id: String,
    publisher_id: Option<String>,
    cover_image: String,
    trailer_url: Option<String>,
    release_date: String,
    tags: Vec<String>,
    platforms: Vec<String>,
    screenshots: Vec<String>,
    price: f64,
    status: String,
    categories: Vec<String>,
    rating_count: i32,
    average_rating: f64,
    purchase_count: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
struct UpdateGameDto {
    name: Option<String>,
    description: Option<String>,
    price: Option<f64>,
    cover_image: Option<String>,
    tags: Option<Vec<String>>,
    platforms: Option<Vec<String>>,
    screenshots: Option<Vec<String>>,
    trailer_url: Option<String>,
    status: Option<String>,
    categories: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ListGamesQuery {
    developer_id: Option<String>,
    categories: Option<Vec<String>>,
    min_price: Option<f64>,
    max_price: Option<f64>,
    status: Option<String>,
    search_query: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
    sort_by: Option<String>,
    sort_desc: Option<bool>,
}

#[derive(Serialize)]
struct ListGamesResponse {
    games: Vec<GameDto>,
    total: i32,
}

#[derive(Deserialize)]
struct DeleteGameDto {
    developer_id: String,
}

struct AppState {
    user_client: user::user_service_client::UserServiceClient<Channel>,
    game_client: game::game_service_client::GameServiceClient<Channel>,
}

async fn create_user(
    data: web::Data<AppState>,
    json: web::Json<CreateUserDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let role = match json.role.as_str() {
        "player" => 0,
        "developer" => 1,
        "admin" => 2,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid role"
            })));
        }
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
                created_at: user
                    .created_at
                    .map(|ts| format!("{}", ts.seconds))
                    .unwrap_or_default(),
            };

            Ok(HttpResponse::Ok().json(user_dto))
        }
        Err(status) => match status.code() {
            tonic::Code::InvalidArgument => {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": status.message()
                })))
            }
            tonic::Code::AlreadyExists => Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "User with this email or username already exists"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            }))),
        },
    }
}

async fn get_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();

    let request = tonic::Request::new(user::GetUserRequest { id: user_id });

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
                    created_at: user
                        .created_at
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
        Err(status) => match status.code() {
            tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            }))),
        },
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
            _ => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid role. Must be: player, developer, or admin"
                })));
            }
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
                        created_at: user
                            .created_at
                            .map(|ts| format!("{}", ts.seconds))
                            .unwrap_or_default(),
                    };
                    Ok(HttpResponse::Ok().json(user_dto))
                }
                None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Server returned empty response"
                }))),
            }
        }
        Err(status) => match status.code() {
            tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }))),
            tonic::Code::InvalidArgument => {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": status.message()
                })))
            }
            tonic::Code::AlreadyExists => Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "Email or username already taken"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Internal error: {}", status.message())
            }))),
        },
    }
}

async fn delete_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();

    let request = tonic::Request::new(user::DeleteUserRequest { id: user_id });

    let mut client = data.user_client.clone();
    match client.delete_user(request).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "User deleted successfully"
        }))),
        Err(status) => match status.code() {
            tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            }))),
        },
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

            let user_dtos: Vec<UserDto> = resp
                .users
                .into_iter()
                .map(|user| UserDto {
                    id: user.id,
                    email: user.email,
                    username: user.username,
                    role: proto_role_to_string(user.role),
                    created_at: user
                        .created_at
                        .map(|ts| format!("{}", ts.seconds))
                        .unwrap_or_default(),
                })
                .collect();

            Ok(HttpResponse::Ok().json(ListUsersHttpResponse {
                users: user_dtos,
                total: resp.total,
            }))
        }
        Err(status) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": status.message()
        }))),
    }
}

async fn create_game(
    data: web::Data<AppState>,
    json: web::Json<CreateGameDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let developer_id = match Uuid::parse_str(&json.developer_id) {
        Ok(uuid) => uuid.to_string(),
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid developer_id format"
            })));
        }
    };

    let request = tonic::Request::new(game::CreateGameRequest {
        name: json.name.clone(),
        description: json.description.clone().unwrap_or_default(),
        developer_id,
        publisher_id: json.publisher_id.clone().unwrap_or_default(),
        cover_image: json.cover_image.clone().unwrap_or_default(),
        trailer_url: json.trailer_url.clone().unwrap_or_default(),
        release_date: json.release_date.clone().unwrap_or_default(),
        tags: json.tags.clone(),
        platforms: json.platforms.clone(),
        screenshots: json.screenshots.clone(),
        price: json.price,
        categories: json.categories.iter().map(|cat| match cat.as_str() {
            "action" => 1,
            "rpg" => 2,
            "strategy" => 3,
            "sports" => 4,
            "racing" => 5,
            "adventure" => 6,
            "simulation" => 7,
            "puzzle" => 8,
            _ => 0, // unspecified
        }).collect(),
    });

    let mut client = data.game_client.clone();
    match client.create_game(request).await {
        Ok(response) => {
            let game = response.into_inner();
            let game_dto = GameDto {
                id: game.id,
                name: game.name,
                description: Some(game.description),
                developer_id: game.developer_id,
                publisher_id: if game.publisher_id.is_empty() { None } else { Some(game.publisher_id) },
                cover_image: game.cover_image,
                trailer_url: if game.trailer_url.is_empty() { None } else { Some(game.trailer_url) },
                release_date: game.release_date,
                tags: game.tags,
                platforms: game.platforms,
                screenshots: game.screenshots,
                price: game.price,
                status: match game.status {
                    0 => "unspecified".to_string(),
                    1 => "draft".to_string(),
                    2 => "under_review".to_string(),
                    3 => "published".to_string(),
                    4 => "suspended".to_string(),
                    _ => "unknown".to_string(),
                },
                categories: game.categories.iter().map(|&cat| match cat {
                    1 => "action".to_string(),
                    2 => "rpg".to_string(),
                    3 => "strategy".to_string(),
                    4 => "sports".to_string(),
                    5 => "racing".to_string(),
                    6 => "adventure".to_string(),
                    7 => "simulation".to_string(),
                    8 => "puzzle".to_string(),
                    _ => "unspecified".to_string(),
                }).collect(),
                rating_count: game.rating_count as i32,
                average_rating: game.average_rating,
                purchase_count: game.purchase_count as i32,
                created_at: game.created_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
                updated_at: game.updated_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
            };
            Ok(HttpResponse::Ok().json(game_dto))
        }
        Err(status) => match status.code() {
            tonic::Code::InvalidArgument => {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": status.message()
                })))
            }
            tonic::Code::AlreadyExists => Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "Game with this name already exists"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            }))),
        },
    }
}

async fn get_game(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let game_id = path.into_inner();

    let request = tonic::Request::new(game::GetGameRequest { id: game_id });

    let mut client = data.game_client.clone();
    match client.get_game(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            if let Some(game) = resp.game {
                let game_dto = GameDto {
                    id: game.id,
                    name: game.name,
                    description: Some(game.description),
                    developer_id: game.developer_id,
                    publisher_id: if game.publisher_id.is_empty() { None } else { Some(game.publisher_id) },
                    cover_image: game.cover_image,
                    trailer_url: if game.trailer_url.is_empty() { None } else { Some(game.trailer_url) },
                    release_date: game.release_date,
                    tags: game.tags,
                    platforms: game.platforms,
                    screenshots: game.screenshots,
                    price: game.price,
                    status: match game.status {
                        0 => "unspecified".to_string(),
                        1 => "draft".to_string(),
                        2 => "under_review".to_string(),
                        3 => "published".to_string(),
                        4 => "suspended".to_string(),
                        _ => "unknown".to_string(),
                    },
                    categories: game.categories.iter().map(|&cat| match cat {
                        1 => "action".to_string(),
                        2 => "rpg".to_string(),
                        3 => "strategy".to_string(),
                        4 => "sports".to_string(),
                        5 => "racing".to_string(),
                        6 => "adventure".to_string(),
                        7 => "simulation".to_string(),
                        8 => "puzzle".to_string(),
                        _ => "unspecified".to_string(),
                    }).collect(),
                    rating_count: game.rating_count as i32,
                    average_rating: game.average_rating,
                    purchase_count: game.purchase_count as i32,
                    created_at: game.created_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
                    updated_at: game.updated_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
                };
                Ok(HttpResponse::Ok().json(game_dto))
            } else {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Game not found"
                })))
            }
        }
        Err(status) => match status.code() {
            tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Game not found"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            }))),
        },
        
    }
}

async fn update_game(
    data: web::Data<AppState>,
    path: web::Path<String>,
    json: web::Json<UpdateGameDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let game_id = path.into_inner();

    if uuid::Uuid::parse_str(&game_id).is_err() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid game ID format"
        })));
    }

    let status = match json.status.as_deref() {
        Some("draft") => Some(1),
        Some("under_review") => Some(2),
        Some("published") => Some(3),
        Some("suspended") => Some(4),
        Some("unspecified") => Some(0),
        None => None,
        Some(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid status. Must be: draft, under_review, published, suspended, or unspecified"
            })));
        }
    };

    let categories = json.categories.as_ref().map(|cats| 
        cats.iter().map(|cat| match cat.as_str() {
            "action" => 1,
            "rpg" => 2,
            "strategy" => 3,
            "sports" => 4,
            "racing" => 5,
            "adventure" => 6,
            "simulation" => 7,
            "puzzle" => 8,
            _ => 0, // unspecified
        }).collect()
    ).unwrap_or_default();

    let request = tonic::Request::new(game::UpdateGameRequest {
        id: game_id,
        name: json.name.clone(),
        description: json.description.clone(),
        price: json.price,
        cover_image: json.cover_image.clone(),
        tags: json.tags.clone().unwrap_or_default(),
        platforms: json.platforms.clone().unwrap_or_default(),
        screenshots: json.screenshots.clone().unwrap_or_default(),
        trailer_url: json.trailer_url.clone(),
        status,
        categories,
    });

    let mut client = data.game_client.clone();
    match client.update_game(request).await {
        Ok(response) => {
            let game = response.into_inner();
            let game_dto = GameDto {
                id: game.id,
                name: game.name,
                description: Some(game.description),
                developer_id: game.developer_id,
                publisher_id: if game.publisher_id.is_empty() { None } else { Some(game.publisher_id) },
                cover_image: game.cover_image,
                trailer_url: if game.trailer_url.is_empty() { None } else { Some(game.trailer_url) },
                release_date: game.release_date,
                tags: game.tags,
                platforms: game.platforms,
                screenshots: game.screenshots,
                price: game.price,
                status: match game.status {
                    0 => "unspecified".to_string(), 
                    1 => "draft".to_string(),
                    2 => "under_review".to_string(),
                    3 => "published".to_string(),
                    4 => "suspended".to_string(),
                    _ => "unknown".to_string(),
                },
                categories: game.categories.iter().map(|&cat| match cat {
                    1 => "action".to_string(),
                    2 => "rpg".to_string(),
                    3 => "strategy".to_string(),
                    4 => "sports".to_string(),
                    5 => "racing".to_string(),
                    6 => "adventure".to_string(),
                    7 => "simulation".to_string(),
                    8 => "puzzle".to_string(),
                    _ => "unspecified".to_string(),
                }).collect(),
                rating_count: game.rating_count as i32,
                average_rating: game.average_rating,
                purchase_count: game.purchase_count as i32,
                created_at: game.created_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
                updated_at: game.updated_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
            };
            Ok(HttpResponse::Ok().json(game_dto))
        }
        Err(status) => match status.code() {
            tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Game not found"
            }))),
            tonic::Code::InvalidArgument => {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": status.message()
                })))
            }
            tonic::Code::PermissionDenied => Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Permission denied: You can only update your own games"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            }))),
        },
    }
}


async fn delete_game(
    data: web::Data<AppState>,
    path: web::Path<String>,
    json: web::Json<DeleteGameDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let game_id = path.into_inner();

    if uuid::Uuid::parse_str(&game_id).is_err() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid game ID format"
        })));
    }

    if uuid::Uuid::parse_str(&json.developer_id).is_err() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid developer_id format"
        })));
    }

    let request = tonic::Request::new(game::DeleteGameRequest {
        id: game_id,
        developer_id: json.developer_id.clone(),
    });

    let mut client = data.game_client.clone();
    match client.delete_game(request).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Game deleted successfully"
        }))),
        Err(status) => match status.code() {
            tonic::Code::NotFound => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Game not found"
            }))),
            tonic::Code::PermissionDenied => Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Permission denied: You can only delete your own games"
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": status.message()
            }))),
        },
    }
}

async fn list_games(
    data: web::Data<AppState>,
    query: web::Query<ListGamesQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let categories = query.categories.as_ref().map(|cats| 
        cats.iter().map(|cat| match cat.as_str() {
            "action" => 1,
            "rpg" => 2,
            "strategy" => 3,
            "sports" => 4,
            "racing" => 5,
            "adventure" => 6,
            "simulation" => 7,
            "puzzle" => 8,
            _ => 0, // unspecified
        }).collect()
    ).unwrap_or_default();

    let status = query.status.as_ref().and_then(|status_str| match status_str.as_str() {
        "draft" => Some(1),
        "under_review" => Some(2),
        "published" => Some(3),
        "suspended" => Some(4),
        "unspecified" => Some(0),
        _ => None,
    });

    let request = tonic::Request::new(game::ListGamesRequest {
        developer_id: query.developer_id.clone(),
        categories,
        min_price: query.min_price,
        max_price: query.max_price,
        status,
        search_query: query.search_query.clone(),
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
        sort_by: query.sort_by.clone(),
        sort_desc: query.sort_desc,
    });

    let mut client = data.game_client.clone();
    match client.list_games(request).await {
        Ok(response) => {
            let resp = response.into_inner();

            let game_dtos: Vec<GameDto> = resp
                .games
                .into_iter()
                .map(|game| GameDto {
                    id: game.id,
                    name: game.name,
                    description: Some(game.description),
                    developer_id: game.developer_id,
                    publisher_id: if game.publisher_id.is_empty() { None } else { Some(game.publisher_id) },
                    cover_image: game.cover_image,
                    trailer_url: if game.trailer_url.is_empty() { None } else { Some(game.trailer_url) },
                    release_date: game.release_date,
                    tags: game.tags,
                    platforms: game.platforms,
                    screenshots: game.screenshots,
                    price: game.price,
                    status: match game.status {
                        0 => "unspecified".to_string(),
                        1 => "draft".to_string(),
                        2 => "under_review".to_string(),
                        3 => "published".to_string(),
                        4 => "suspended".to_string(),
                        _ => "unknown".to_string(),
                    },
                    categories: game.categories.iter().map(|&cat| match cat {
                        1 => "action".to_string(),
                        2 => "rpg".to_string(),
                        3 => "strategy".to_string(),
                        4 => "sports".to_string(),
                        5 => "racing".to_string(),
                        6 => "adventure".to_string(),
                        7 => "simulation".to_string(),
                        8 => "puzzle".to_string(),
                        _ => "unspecified".to_string(),
                    }).collect(),
                    rating_count: game.rating_count as i32,
                    average_rating: game.average_rating,
                    purchase_count: game.purchase_count as i32,
                    created_at: game.created_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
                    updated_at: game.updated_at.map(|ts| format!("{}", ts.seconds)).unwrap_or_default(),
                })
                .collect();

            Ok(HttpResponse::Ok().json(ListGamesResponse {
                games: game_dtos,
                total: resp.total,
            }))
        }
        Err(status) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": status.message()
        }))),
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
    let ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if !rate_limiter.check_rate_limit(&ip, 100, Duration::from_secs(60)) {
        return Ok(req.into_response(
            HttpResponse::TooManyRequests()
                .json(serde_json::json!({
                    "error": "Rate limit exceeded. Please try again later."
                }))
                .map_into_boxed_body(),
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

    println!(
        "Request ID: {} - {} {}",
        request_id,
        req.method(),
        req.path()
    );

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

    let game_client = game::game_service_client::GameServiceClient::connect("http://[::1]:50052")
        .await
        .expect("Failed to connect to game service");

    let app_state = web::Data::new(AppState { user_client, game_client });

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
            .wrap(middleware::Logger::new(
                "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
            ))
            .route("/api/users", web::post().to(create_user))
            .route("/api/users/{id}", web::get().to(get_user))
            .route("/api/users/{id}", web::put().to(update_user))
            .route("/api/users/{id}", web::delete().to(delete_user))
            .route("/api/users", web::get().to(users_list))
            .route("/api/games", web::post().to(create_game))
            .route("/api/games/{id}", web::get().to(get_game))
            .route("/api/games/{id}", web::put().to(update_game))
            .route("/api/games/{id}", web::delete().to(delete_game))
            .route("/api/games", web::get().to(list_games))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
