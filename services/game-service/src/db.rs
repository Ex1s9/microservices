use chrono::{DateTime, NaiveDate, Utc};
use sqlx::postgres::PgPool;
use sqlx::types::Decimal;
use uuid::Uuid;

use crate::models::{DbGame, DbGameCategory, DbGameStatus};

pub async fn create_game(
     pool: &PgPool,
     name: String,
     description: String,
     developer_id: Uuid,
     publisher_id: Option<Uuid>,
     cover_image: String,
     trailer_url: Option<String>,
     release_date: NaiveDate,
     categories: Vec<DbGameCategory>,
     tags: Vec<String>,
     platforms: Vec<String>,
     price: Decimal,
) -> Result<DbGame, sqlx::Error> {
     let id = Uuid::new_v4();
     let now = Utc::now();

     let game = sqlx::query_as!(
          DbGame,
          r#"
          INSERT INTO games (
               id, name, description, developer_id, publisher_id, 
               cover_image, trailer_url, release_date, price, status,
               categories, tags, platforms, screenshots,
               created_at, updated_at
          )
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::game_status, $11::game_category[], $12, $13, $14, $15, $15)
          RETURNING 
               id, name, description, developer_id, publisher_id,
               cover_image, trailer_url, release_date, price, 
               status as "status: DbGameStatus",
               categories as "categories: Vec<DbGameCategory>",
               tags, platforms, screenshots, 
               rating_count, average_rating, purchase_count,
               created_at, updated_at, deleted_at
          "#,
          id,
          name,
          description,
          developer_id,
          publisher_id,
          cover_image,
          trailer_url,
          release_date,
          price,
          DbGameStatus::Draft,
          &categories,
          &tags,
          &platforms,
          &Vec::<String>::new(),
          now
     )
     .fetch_one(pool)
     .await?;

     Ok(game)
}

pub async fn get_game_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DbGame>, sqlx::Error> {
     let record = sqlx::query_as!(
          DbGame,
          r#"
          SELECT 
               id, name, description, developer_id, publisher_id,
               cover_image, trailer_url, release_date, price, 
               status as "status: DbGameStatus",
               categories as "categories: Vec<DbGameCategory>",
               tags, platforms, screenshots,
               rating_count, average_rating, purchase_count,
               created_at, updated_at, deleted_at
          FROM games
          WHERE id = $1 AND deleted_at IS NULL
          "#,
          id
     )
     .fetch_optional(pool)
     .await?;

     Ok(record)
}

pub async fn update_game(
     pool: &PgPool,
     id: Uuid,
     name: Option<String>,
     description: Option<String>,
     price: Option<Decimal>,
     cover_image: Option<String>,
     trailer_url: Option<String>,
     status: Option<DbGameStatus>,
     categories: Option<Vec<DbGameCategory>>,
     tags: Option<Vec<String>>,
     platforms: Option<Vec<String>>,
     screenshots: Option<Vec<String>>,
) -> Result<DbGame, sqlx::Error> {
     let now = Utc::now();

     let record = sqlx::query_as!(
          DbGame,
          r#"
          UPDATE games
          SET 
               name = COALESCE($2, name),
               description = COALESCE($3, description),
               price = COALESCE($4, price),
               cover_image = COALESCE($5, cover_image),
               trailer_url = COALESCE($6, trailer_url),
               status = COALESCE($7::game_status, status),
               categories = COALESCE($8::game_category[], categories),
               tags = COALESCE($9, tags),
               platforms = COALESCE($10, platforms),
               screenshots = COALESCE($11, screenshots),
               updated_at = $12
          WHERE id = $1 AND deleted_at IS NULL
          RETURNING 
               id, name, description, developer_id, publisher_id,
               cover_image, trailer_url, release_date, price, 
               status as "status: DbGameStatus",
               categories as "categories: Vec<DbGameCategory>",
               tags, platforms, screenshots,
               rating_count, average_rating, purchase_count,
               created_at, updated_at, deleted_at
          "#,
          id,
          name,
          description,
          price,
          cover_image,
          trailer_url,
          status,
          categories.as_deref() as Option<&[DbGameCategory]>,
          tags.as_deref(),
          platforms.as_deref(),
          screenshots.as_deref(),
          now
     )
     .fetch_one(pool)
     .await?;

     Ok(record)
}

pub async fn delete_game(pool: &PgPool, id: Uuid, developer_id: Uuid) -> Result<bool, sqlx::Error> {
     let now = Utc::now();
     let rows_affected = sqlx::query!(
          r#"
          UPDATE games 
          SET deleted_at = $3
          WHERE id = $1 AND developer_id = $2 AND deleted_at IS NULL
          "#,
          id,
          developer_id,
          now
     )
     .execute(pool)
     .await?
     .rows_affected();

     Ok(rows_affected > 0)
}

pub async fn get_all_games(pool: &PgPool) -> Result<Vec<DbGame>, sqlx::Error> {
     let records = sqlx::query_as!(
          DbGame,
          r#"
          SELECT 
               id, name, description, developer_id, publisher_id,
               cover_image, trailer_url, release_date, price, 
               status as "status: DbGameStatus",
               categories as "categories: Vec<DbGameCategory>",
               tags, platforms, screenshots,
               rating_count, average_rating, purchase_count,
               created_at, updated_at, deleted_at
          FROM games
          WHERE deleted_at IS NULL
          ORDER BY created_at DESC
          "#
     )
     .fetch_all(pool)
     .await?;
     
     Ok(records) 
}

pub async fn list_games(
     pool: &PgPool,
     developer_id: Option<Uuid>,
     categories: Option<Vec<DbGameCategory>>,
     min_price: Option<Decimal>,
     max_price: Option<Decimal>,
     status: Option<DbGameStatus>,
     search_query: Option<String>,
     limit: i32,
     offset: i32,
) -> Result<(Vec<DbGame>, i64), sqlx::Error> {
     let games = sqlx::query_as!(
          DbGame,
          r#"
          SELECT 
               id, name, description, developer_id, publisher_id,
               cover_image, trailer_url, release_date, price, 
               status as "status: DbGameStatus",
               categories as "categories: Vec<DbGameCategory>",
               tags, platforms, screenshots,
               rating_count, average_rating, purchase_count,
               created_at, updated_at, deleted_at
          FROM games
          WHERE deleted_at IS NULL
               AND ($1::uuid IS NULL OR developer_id = $1)
               AND ($2::game_category[] IS NULL OR categories && $2)
               AND ($3::decimal IS NULL OR price >= $3)
               AND ($4::decimal IS NULL OR price <= $4)  
               AND ($5::game_status IS NULL OR status = $5)
               AND ($6::text IS NULL OR to_tsvector('english', name) @@ plainto_tsquery('english', $6))
          ORDER BY created_at DESC
          LIMIT $7 OFFSET $8
          "#,
          developer_id,
          categories.as_deref() as Option<&[DbGameCategory]>,
          min_price,
          max_price,
          status as Option<DbGameStatus>,
          search_query,
          limit as i64,
          offset as i64
     )
     .fetch_all(pool)
     .await?;

     let total = sqlx::query_scalar!(
          r#"
          SELECT COUNT(*) FROM games 
          WHERE deleted_at IS NULL
               AND ($1::uuid IS NULL OR developer_id = $1)
               AND ($2::game_category[] IS NULL OR categories && $2)
               AND ($3::decimal IS NULL OR price >= $3)
               AND ($4::decimal IS NULL OR price <= $4)  
               AND ($5::game_status IS NULL OR status = $5)
               AND ($6::text IS NULL OR to_tsvector('english', name) @@ plainto_tsquery('english', $6))
          "#,
          developer_id,
          categories.as_deref() as Option<&[DbGameCategory]>,
          min_price,
          max_price,
          status as Option<DbGameStatus>,
          search_query
     )
     .fetch_one(pool)
     .await?
     .unwrap_or(0);

     Ok((games, total))
}

pub async fn get_games_by_category(
     pool: &PgPool,
     category: DbGameCategory,
     limit: i32,
     offset: i32,
) -> Result<Vec<DbGame>, sqlx::Error> {
     let games = sqlx::query_as!(
          DbGame,
          r#"
          SELECT 
               id, name, description, developer_id, publisher_id,
               cover_image, trailer_url, release_date, price, 
               status as "status: DbGameStatus",
               categories as "categories: Vec<DbGameCategory>",
               tags, platforms, screenshots,
               rating_count, average_rating, purchase_count,
               created_at, updated_at, deleted_at
          FROM games
          WHERE $1::game_category = ANY(categories) 
               AND status = 'published'::game_status 
               AND deleted_at IS NULL
          ORDER BY average_rating DESC, purchase_count DESC
          LIMIT $2 OFFSET $3
          "#,
          category as DbGameCategory,
          limit as i64,
          offset as i64
     )
     .fetch_all(pool)
     .await?;

     Ok(games)
}

pub async fn get_popular_games(
     pool: &PgPool,
     limit: i32,
) -> Result<Vec<DbGame>, sqlx::Error> {
     let games = sqlx::query_as!(
          DbGame,
          r#"
          SELECT 
               id, name, description, developer_id, publisher_id,
               cover_image, trailer_url, release_date, price, 
               status as "status: DbGameStatus",
               categories as "categories: Vec<DbGameCategory>",
               tags, platforms, screenshots,
               rating_count, average_rating, purchase_count,
               created_at, updated_at, deleted_at
          FROM games
          WHERE status = 'published'::game_status AND deleted_at IS NULL
          ORDER BY purchase_count DESC, average_rating DESC
          LIMIT $1
          "#,
          limit as i64
     )
     .fetch_all(pool)
     .await?;

     Ok(games)
}

pub async fn update_game_rating(
     pool: &PgPool,
     game_id: Uuid,
     new_rating: Decimal,
) -> Result<(), sqlx::Error> {
     sqlx::query!(
          r#"
          UPDATE games
          SET 
               average_rating = (
                    (average_rating * rating_count + $2) / (rating_count + 1)
               ),
               rating_count = rating_count + 1,
               updated_at = NOW()
          WHERE id = $1 AND deleted_at IS NULL
          "#,
          game_id,
          new_rating
     )
     .execute(pool)
     .await?;

     Ok(())
}

pub async fn increment_purchase_count(
     pool: &PgPool,
     game_id: Uuid,
) -> Result<(), sqlx::Error> {
     sqlx::query!(
          r#"
          UPDATE games
          SET 
               purchase_count = purchase_count + 1,
               updated_at = NOW()
          WHERE id = $1 AND deleted_at IS NULL
          "#,
          game_id
     )
     .execute(pool)
     .await?;

     Ok(())
}

pub async fn add_screenshot(
     pool: &PgPool,
     game_id: Uuid,
     screenshot_url: String,
) -> Result<(), sqlx::Error> {
     sqlx::query!(
          r#"
          UPDATE games
          SET 
               screenshots = array_append(screenshots, $2),
               updated_at = NOW()
          WHERE id = $1 AND deleted_at IS NULL
          "#,
          game_id,
          screenshot_url
     )
     .execute(pool)
     .await?;

     Ok(())
}

pub async fn remove_screenshot(
     pool: &PgPool,
     game_id: Uuid,
     screenshot_url: String,
) -> Result<(), sqlx::Error> {
     sqlx::query!(
          r#"
          UPDATE games
          SET 
               screenshots = array_remove(screenshots, $2),
               updated_at = NOW()
          WHERE id = $1 AND deleted_at IS NULL
          "#,
          game_id,
          screenshot_url
     )
     .execute(pool)
     .await?;

     Ok(())
}