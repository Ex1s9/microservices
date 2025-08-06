use axum::{
    routing::post,
    Router,
};
use tower_http::cors::CorsLayer;

use crate::handlers::create_game_http;

pub fn create_routes() -> Router {
    Router::new()
        .route("/api/games", post(create_game_http))
        .layer(CorsLayer::permissive())
}