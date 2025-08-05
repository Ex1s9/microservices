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
     Unspecified,
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
     pub categories: Vec<DbGameCategory>,
     pub tags: Vec<String>,
     pub platforms: Vec<String>,
     pub screenshots: Vec<String>,
     pub rating_count: i32,
     pub average_rating: Decimal,
     pub purchase_count: i32,
     pub created_at: DateTime<Utc>,
     pub updated_at: DateTime<Utc>,
     pub deleted_at: Option<DateTime<Utc>>,
}

impl DbGameCategory {
     pub fn from_proto(value: i32) -> Self {
          match value {
               1 => Self::Action,
               2 => Self::Rpg,
               3 => Self::Strategy,
               4 => Self::Sports,
               5 => Self::Racing,
               6 => Self::Adventure,
               7 => Self::Simulation,
               8 => Self::Puzzle,
               _ => Self::Unspecified,
          }
     }

     pub fn to_proto(&self) -> i32 {
          match self {
               Self::Action => 1,
               Self::Rpg => 2,
               Self::Strategy => 3,
               Self::Sports => 4,
               Self::Racing => 5,
               Self::Adventure => 6,
               Self::Simulation => 7,
               Self::Puzzle => 8,
               Self::Unspecified => 0,
          }
     }
}

impl DbGameStatus {
     pub fn from_proto(value: i32) -> Self {
          match value {
               1 => Self::Draft,
               2 => Self::UnderReview,
               3 => Self::Published,
               4 => Self::Suspended,
               _ => Self::Unspecified,
          }
     }

     pub fn to_proto(&self) -> i32 {
          match self {
               Self::Draft => 1,
               Self::UnderReview => 2,
               Self::Published => 3,
               Self::Suspended => 4,
               Self::Unspecified => 0,
          }
     }
}