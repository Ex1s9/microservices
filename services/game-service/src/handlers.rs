use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
};
use sqlx::PgPool;
use tonic::Request;

use crate::game;
use crate::grpc_service::GameServiceImpl;
use crate::types::{CreateGameRequest, GameResponse};

pub async fn create_game_http(
    State(pool): State<PgPool>,
    Json(request): Json<CreateGameRequest>,
) -> Result<ResponseJson<GameResponse>, StatusCode> {
    use crate::game::game_service_server::GameService;
    
    let service = GameServiceImpl { pool };
    
    let grpc_request = game::CreateGameRequest {
        name: request.name,
        description: request.description,
        developer_id: request.developer_id,
        publisher_id: request.publisher_id,
        cover_image: request.cover_image,
        trailer_url: request.trailer_url,
        release_date: request.release_date,
        categories: request.categories,
        tags: request.tags,
        platforms: request.platforms,
        price: request.price as i64,
    };

    match service.create_game(Request::new(grpc_request)).await {
        Ok(response) => {
            let game_response = service.convert_to_response(response.into_inner());
            Ok(ResponseJson(game_response))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}