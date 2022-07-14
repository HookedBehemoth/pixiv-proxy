use super::{
    de::deserialize_number_unconditionally,
    error::ApiError,
    fetch::{fetch_json, post_and_fetch_json},
};

use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Deserialize, Serialize)]
pub struct SketchImage {
    width: u32,
    height: u32,
    url: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchShortMedia {
    url: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchPhotos {
    original: SketchImage,
    pxw540: SketchImage,
    pxsq60: SketchImage,
}

#[derive(Deserialize, Serialize)]
pub struct SketchMedia {
    photo: SketchPhotos,
}

#[derive(Deserialize, Serialize)]
pub struct SketchTextFragment {
    #[serde(rename = "type")]
    t: String,
    body: String,
    normalized_body: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserAccount {
    unique_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserAccounts {
    twitter: Option<SketchUserAccount>,
    pixiv: Option<SketchUserAccount>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserStats {
    follower_count: u32,
    following_count: u32,
    heart_count: u32,
    resnap_count: u32,
    public_post_count: u32,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUser {
    id: u64,
    pixiv_user_id: u64,
    name: String,
    description_fragments: Option<Vec<SketchTextFragment>>,
    icon: SketchMedia,
    social_accounts: Option<SketchUserAccounts>,
    stats: Option<SketchUserStats>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserPosts {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    key: u64,
    user: SketchUser,
    posts: Vec<SketchItem>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchItem {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    id: u64,
    comment_count: u32,
    user: SketchUser,
    is_r18: bool,
    media: Vec<SketchMedia>,
    tags: Vec<String>,
    text_fragments: Vec<SketchTextFragment>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchImpressions {
    feedbacks: Vec<SketchImpression>,
    item: SketchItem,
}

#[derive(Deserialize, Serialize)]
pub struct SketchImpression {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    id: u64,
    #[serde(rename = "type")]
    t: String,
    user: SketchUser,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchLives {
    lives: Vec<SketchLive>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchLive {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    id: u64,
    created_at: chrono::DateTime<chrono::Utc>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
    user: SketchUser,
    name: String,
    description_fragments: Vec<SketchTextFragment>,
    is_r18: bool,
    is_broadcasting: bool,
    audience_count: u32,
    total_audience_count: u32,
    heart_count: u32,
    chat_count: u32,
}

#[derive(Deserialize, Serialize)]
pub struct SketchLiveOwner {
    hls_movie: SketchShortMedia,
}

#[derive(Deserialize, Serialize)]
pub struct SketchApiResponse<T> {
    data: T,
}

// https://sketch.pixiv.net/api/items/3455710150565207900
pub async fn fetch_item(
    client: &awc::Client,
    id: u64,
) -> Result<SketchApiResponse<SketchItem>, ApiError> {
    let url = format!("https://sketch.pixiv.net/api/items/{id}");
    fetch_json(client, &url).await
}

// https://sketch.pixiv.net/api/feedbacks/3455710150565207900
pub async fn fetch_feedbacks(
    client: &awc::Client,
    id: u64,
) -> Result<SketchApiResponse<SketchImpressions>, ApiError> {
    let url = format!("https://sketch.pixiv.net/api/feedbacks/{id}");
    fetch_json(client, &url).await
}

// https://sketch.pixiv.net/api/users/51062509
pub async fn fetch_user(
    client: &awc::Client,
    id: u64,
) -> Result<SketchApiResponse<SketchUser>, ApiError> {
    let url = format!("https://sketch.pixiv.net/api/users/{id}");
    fetch_json(client, &url).await
}

// https://sketch.pixiv.net/api/users/posts/latest.json
pub async fn fetch_latest_user_posts(
    client: &awc::Client,
    ids: &[u64],
    count: u32,
) -> Result<SketchApiResponse<SketchUserPosts>, ApiError> {
    if count > 6 {
        return Err(ApiError::External(400, "count is too big".to_owned()));
    }

    let url = "https://sketch.pixiv.net/api/users/posts/latest.json";
    let mut form = format!("count={count}");
    for id in ids {
        let _ = write!(form, "&users%5B%5D={id}");
    }
    post_and_fetch_json(client, url, form).await
}

// https://sketch.pixiv.net/api/lives.json?count=20&order_by=audience_count
pub async fn fetch_lives(
    client: &awc::Client,
    count: u32,
    order_by: &str,
) -> Result<SketchApiResponse<SketchLives>, ApiError> {
    let url = format!("https://sketch.pixiv.net/api/lives.json?count={count}&order_by={order_by}");
    fetch_json(client, &url).await
}
