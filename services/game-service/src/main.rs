use tonic::transport::Server;
use tonic::{Request, Response, Status};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use dotenv::dotenv;
use std::env;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;

use uuid::Uuid;

// use error::GameServiceError;
use models::{DbGameCategory, DbGameStatus};
use rust_decimal::Decimal;

pub mod game {
    tonic::include_proto!("game");
}

mod db;
// mod error;
// mod validation;
mod models;

pub struct GameServiceImpl {
    pool: PgPool,
}

impl GameServiceImpl {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl game::game_service_server::GameService for GameServiceImpl {
    async fn create_game(
        &self,
        request: Request<game::CreateGameRequest>,
    ) -> Result<Response<game::Game>, Status> {
        let req = request.into_inner();

        let game_record = db::create_game(
            &self.pool,
            req.name,
            req.description,
            Uuid::parse_str(&req.developer_id).map_err(|_| Status::invalid_argument("Invalid developer_id"))?,
            req.publisher_id.as_ref().map(|id| Uuid::parse_str(id)).transpose().map_err(|_| Status::invalid_argument("Invalid publisher_id"))?,
            req.cover_image,
            req.trailer_url,
            chrono::NaiveDate::parse_from_str(&req.release_date, "%Y-%m-%d").map_err(|_| Status::invalid_argument("Invalid date format"))?,
            req.categories.into_iter().map(DbGameCategory::from_proto).collect(),
            req.tags,
            req.platforms,
            Decimal::from_f64_retain(req.price).unwrap_or_default(),
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to create game: {}", e)))?;
        
        let game_msg = game::Game {
            id: game_record.id.to_string(),
            name: game_record.name,
            description: Some(game_record.description),
            developer_id: game_record.developer_id.to_string(),
            publisher_id: game_record.publisher_id.map(|id| id.to_string()),
            cover_image: game_record.cover_image,
            trailer_url: game_record.trailer_url,
            release_date: game_record.release_date.to_string(),
            tags: game_record.tags,
            platforms: game_record.platforms,
            screenshots: game_record.screenshots,
            price: game_record.price.to_f64().unwrap_or(0.0),
            created_at: Some(Timestamp {
                seconds: game_record.created_at.timestamp(),
                nanos: game_record.created_at.timestamp_subsec_nanos() as i32,
            }),
            updated_at: Some(Timestamp {
                seconds: game_record.updated_at.timestamp(),
                nanos: game_record.updated_at.timestamp_subsec_nanos() as i32,
            }),
            status: game_record.status.to_proto(),
            categories: game_record.categories.into_iter().map(|c| c.to_proto()).collect(),
            rating_count: game_record.rating_count,
            average_rating: game_record.average_rating.to_f64().unwrap_or(0.0),
            purchase_count: game_record.purchase_count,
        };

        Ok(Response::new(game_msg))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let addr = "[::1]:50051".parse()?;
    let game_service = GameServiceImpl::new(pool);

    println!("Game service listening on {}", addr);

    Server::builder()
        .add_service(game::game_service_server::GameServiceServer::new(
            game_service,
        ))
        .serve(addr)
        .await?;

    Ok(())
}