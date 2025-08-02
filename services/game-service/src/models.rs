use chrono::{DateTime, Utc};
use sqlx::types::Decimal;
use uuid::Uuid;

#[derive(Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "game_category", rename_all = "lowercase")]
pub enum DbGameCategory {
     Unspecified,
     Action,
     Rpg,
     Strategy,
     Sports,
     Racing,
     Adventure,
     Simulation,
     Puzzle,
}

#[derive(Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "game_status", rename_all = "snake_case")]
pub enum DbGameStatus {
     Draft,
     UnderReview,
     Published,
     Suspended,
}

#[derive(Debug, Clone)]
pub struct DbGame {
     pub id: Uuid,
     pub name: String,
     pub description: String,
     pub developer_id: Uuid,
     pub publisher_id: Option<Uuid>,
     pub cover_image: String,
     pub trailer_url: Option<String>,
     pub release_date: chrono::NaiveDate,
     pub price: Decimal,
     pub status: DbGameStatus,
     pub rating_count: i32,
     pub average_rating: Decimal,
     pub purchase_count: i32,
     pub created_at: DateTime<Utc>,
     pub updated_at: DateTime<Utc>,
     pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct DbGameFull {
     pub game: DbGame,
     pub categories: Vec<DbGameCategory>,
     pub tags: Vec<String>,
     pub platforms: Vec<String>,
     pub screenshots: Vec<String>,
}