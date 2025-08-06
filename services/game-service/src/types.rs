use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub name: String,
    pub description: String,
    pub developer_id: String,
    pub release_date: String,
    pub categories: Vec<i32>,
    pub tags: Vec<String>,
    pub platforms: Vec<String>,
    pub price: f64,
    pub cover_image: String,
    pub publisher_id: Option<String>,
    pub trailer_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub developer_id: String,
    pub publisher_id: Option<String>,
    pub cover_image: String,
    pub trailer_url: Option<String>,
    pub release_date: String,
    pub tags: Vec<String>,
    pub platforms: Vec<String>,
    pub screenshots: Vec<String>,
    pub price: f64,
    pub status: String,
    pub categories: Vec<String>,
    pub rating_count: i32,
    pub average_rating: f64,
    pub purchase_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}