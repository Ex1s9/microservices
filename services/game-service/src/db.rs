
pub async fn create_game(pool: &PgPool, game: &DbGame) -> Result<DbGame, sqlx::Error> {
     let now = Utc::now();

     let record = aqlx::query_as!(
          DbGame,
          r#"
          INSERT INTO games (id, name, description, category, status, price, created_at, updated_at)
          VALUES ($1, $2, $3, $4::game_category, $5::game_status, $6, $7, $7)
          RETURNING id, name, description, category as "category: DbGameCategory", status as "status: DbGameStatus", price, created_at, updated_at
          "#,
          game.id,
          game.name,
          game.description,
          game.category as DbGameCategory,
          game.status as DbGameStatus,
          game.price,
          now
     )
     .fetch_one(pool)
     .await?;

     Ok(DbGame {
          id: record.id,
          name: record.name,
          description: record.description,
          category: record.category,
          status: record.status,
          price: record.price,
          created_at: record.created_at,
          updated_at: record.updated_at,
     })
}

