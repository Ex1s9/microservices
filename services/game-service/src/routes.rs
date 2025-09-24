use axum::{
    routing::post,
    Router,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

use crate::handlers::create_game_http;

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/api/games", post(create_game_http))
        .layer(CorsLayer::permissive())
        .with_state(pool)
}