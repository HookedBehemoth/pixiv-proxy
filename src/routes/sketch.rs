use crate::api::{
    error::ApiError,
    sketch::{fetch_feedbacks, fetch_item, fetch_latest_user_posts, fetch_lives, fetch_user},
};

use actix_web::{dev::HttpServiceFactory, get, web, Result};

pub fn routes() -> impl HttpServiceFactory {
    (sketch_user, sketch_item, sketch_impressions, sketch_lives)
}

#[get("/sketch/users/{id}")]
async fn sketch_user(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
) -> Result<String, ApiError> {
    let user = fetch_user(&client, *id).await?;

    Ok(serde_json::to_string_pretty(&user).unwrap())
}

#[get("/sketch/items/{id}")]
async fn sketch_item(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
) -> Result<String, ApiError> {
    let item = fetch_item(&client, *id).await?;

    Ok(serde_json::to_string_pretty(&item).unwrap())
}

#[get("/sketch/lives")]
async fn sketch_lives(client: web::Data<awc::Client>) -> Result<String, ApiError> {
    let lives = fetch_lives(&client, 20, "audience_count").await?;

    Ok(serde_json::to_string_pretty(&lives).unwrap())
}

#[get("/sketch/impressions/{id}")]
async fn sketch_impressions(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
) -> Result<String, ApiError> {
    let impressions = fetch_feedbacks(&client, *id).await?;

    Ok(serde_json::to_string_pretty(&impressions).unwrap())
}
