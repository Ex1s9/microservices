use tonic::{Request, Response, Status};
use uuid::Uuid;
use chrono::Utc;

use crate::game;
use crate::types::GameResponse;

#[derive(Clone)]
pub struct GameServiceImpl;

#[tonic::async_trait]
impl game::game_service_server::GameService for GameServiceImpl {
    async fn create_game(
        &self,
        request: Request<game::CreateGameRequest>,
    ) -> Result<Response<game::Game>, Status> {
        let req = request.into_inner();
        
        let game_msg = game::Game {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            description: Some(req.description),
            developer_id: req.developer_id,
            publisher_id: req.publisher_id,
            cover_image: req.cover_image,
            trailer_url: req.trailer_url,
            release_date: req.release_date,
            tags: req.tags,
            platforms: req.platforms,
            screenshots: vec![],
            price: req.price,
            created_at: Some(prost_types::Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: 0,
            }),
            updated_at: Some(prost_types::Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: 0,
            }),
            status: 1, // Draft
            categories: req.categories,
            rating_count: 0,
            average_rating: 0.0,
            purchase_count: 0,
        };

        Ok(Response::new(game_msg))
    }

    async fn get_game(
        &self,
        _request: Request<game::GetGameRequest>,
    ) -> Result<Response<game::GetGameResponse>, Status> {
        Err(Status::unimplemented("GetGame not implemented yet"))
    }

    async fn update_game(
        &self,
        _request: Request<game::UpdateGameRequest>,
    ) -> Result<Response<game::Game>, Status> {
        Err(Status::unimplemented("UpdateGame not implemented yet"))
    }

    async fn delete_game(
        &self,
        _request: Request<game::DeleteGameRequest>,
    ) -> Result<Response<game::DeleteGameResponse>, Status> {
        Err(Status::unimplemented("DeleteGame not implemented yet"))
    }

    async fn list_games(
        &self,
        _request: Request<game::ListGamesRequest>,
    ) -> Result<Response<game::ListGamesResponse>, Status> {
        Err(Status::unimplemented("ListGames not implemented yet"))
    }
}

impl GameServiceImpl {
    pub fn convert_to_response(&self, game: game::Game) -> GameResponse {
        GameResponse {
            id: game.id,
            name: game.name,
            description: game.description,
            developer_id: game.developer_id,
            publisher_id: game.publisher_id,
            cover_image: game.cover_image,
            trailer_url: game.trailer_url,
            release_date: game.release_date,
            tags: game.tags,
            platforms: game.platforms,
            screenshots: game.screenshots,
            price: game.price,
            status: match game.status {
                1 => "draft".to_string(),
                2 => "under_review".to_string(),
                3 => "published".to_string(),
                4 => "suspended".to_string(),
                _ => "unknown".to_string(),
            },
            categories: game.categories.into_iter().map(|c| match c {
                1 => "action".to_string(),
                2 => "rpg".to_string(),
                3 => "strategy".to_string(),
                4 => "sports".to_string(),
                5 => "racing".to_string(),
                6 => "adventure".to_string(),
                7 => "simulation".to_string(),
                8 => "puzzle".to_string(),
                _ => "unknown".to_string(),
            }).collect(),
            rating_count: game.rating_count,
            average_rating: game.average_rating,
            purchase_count: game.purchase_count,
            created_at: game.created_at.map(|t| format!("{}Z", chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default().format("%Y-%m-%dT%H:%M:%S"))).unwrap_or_default(),
            updated_at: game.updated_at.map(|t| format!("{}Z", chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default().format("%Y-%m-%dT%H:%M:%S"))).unwrap_or_default(),
        }
    }
}