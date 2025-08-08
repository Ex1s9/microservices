use tonic::{Request, Response, Status};
use uuid::Uuid;
use chrono::Utc;
use sqlx::PgPool;

use crate::game;
use crate::types::GameResponse;
use crate::models::{DbGame, DbGameCategory, DbGameStatus};
use crate::db;

#[derive(Clone)]
pub struct GameServiceImpl {
    pub pool: PgPool,
}

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
        request: Request<game::ListGamesRequest>,
    ) -> Result<Response<game::ListGamesResponse>, Status> {
        let req = request.into_inner();

        let limit = req.page_size.max(1).min(100) as i32;
        let offset = req.page_token.parse::<i32>().unwrap_or(0);
        
        let developer_id = if req.developer_id.is_empty() {
            None
        } else {
            Some(Uuid::parse_str(&req.developer_id).map_err(|_| Status::invalid_argument("Invalid developer_id"))?)
        };
        
        let categories: Option<Vec<DbGameCategory>> = if req.categories.is_empty() {
            None
        } else {
            Some(req.categories.into_iter().map(DbGameCategory::from_proto).collect())
        };
        
        let status = if req.status == 0 { None } else { Some(DbGameStatus::from_proto(req.status)) };
        
        let search_query = if req.search_query.is_empty() { None } else { Some(req.search_query) };

        let (db_games, total) = db::list_games(
            &self.pool,
            developer_id,
            categories,
            req.min_price.map(|p| sqlx::types::Decimal::new(p, 2)),
            req.max_price.map(|p| sqlx::types::Decimal::new(p, 2)),
            status,
            search_query,
            limit,
            offset,
        ).await.map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let games: Vec<game::Game> = db_games.into_iter().map(|g| self.db_game_to_proto(g)).collect();
        
        let next_page_token = if (offset + limit) < total as i32 {
            (offset + limit).to_string()
        } else {
            String::new()
        };

        let response = game::ListGamesResponse {
            games,
            total_count: total as u64,
            next_page_token,
        };

        Ok(Response::new(response))
    }
}

impl GameServiceImpl {
    pub fn db_game_to_proto(&self, db_game: DbGame) -> game::Game {
        game::Game {
            id: db_game.id.to_string(),
            name: db_game.name,
            description: Some(db_game.description),
            developer_id: db_game.developer_id.to_string(),
            publisher_id: db_game.publisher_id.map(|p| p.to_string()).unwrap_or_default(),
            cover_image: Some(db_game.cover_image),
            trailer_url: db_game.trailer_url,
            release_date: Some(db_game.release_date.format("%Y-%m-%d").to_string()),
            tags: db_game.tags,
            platforms: db_game.platforms,
            screenshots: db_game.screenshots,
            price: db_game.price.to_string().parse::<i64>().unwrap_or(0),
            created_at: Some(prost_types::Timestamp {
                seconds: db_game.created_at.timestamp(),
                nanos: (db_game.created_at.timestamp_subsec_nanos()) as i32,
            }),
            updated_at: Some(prost_types::Timestamp {
                seconds: db_game.updated_at.timestamp(),
                nanos: (db_game.updated_at.timestamp_subsec_nanos()) as i32,
            }),
            status: db_game.status.to_proto(),
            categories: db_game.categories.into_iter().map(|c| c.to_proto()).collect(),
            rating_count: db_game.rating_count as u32,
            average_rating: db_game.average_rating.to_string().parse::<f64>().unwrap_or(0.0),
            purchase_count: db_game.purchase_count as u32,
        }
    }

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