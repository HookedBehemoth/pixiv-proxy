use super::{
    de::{deserialize_number_unconditionally, strip_url_prefix},
    error::ApiError,
    fetch::{fetch_json, post_and_fetch_json},
};

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Write};

#[derive(Deserialize, Serialize)]
pub struct SketchImage {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "strip_url_prefix")]
    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchShortMedia {
    #[serde(deserialize_with = "strip_url_prefix")]
    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchPhotos {
    pub original: SketchImage,
    pub pxw540: SketchImage,
    pub pxsq60: SketchImage,
}

#[derive(Deserialize, Serialize)]
pub struct SketchMedia {
    pub photo: SketchPhotos,
}

#[derive(Deserialize, Serialize)]
pub struct SketchTextFragment {
    #[serde(rename = "type")]
    pub t: String,
    pub body: String,
    pub normalized_body: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserAccount {
    pub unique_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserAccounts {
    pub twitter: Option<SketchUserAccount>,
    pub pixiv: Option<SketchUserAccount>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserStats {
    pub follower_count: u32,
    pub following_count: u32,
    pub heart_count: u32,
    pub resnap_count: u32,
    pub public_post_count: u32,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUser {
    pub id: u64,
    pub pixiv_user_id: u64,
    pub name: String,
    pub description_fragments: Option<Vec<SketchTextFragment>>,
    pub icon: SketchMedia,
    pub social_accounts: Option<SketchUserAccounts>,
    pub stats: Option<SketchUserStats>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchUserPosts {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    pub key: u64,
    pub user: SketchUser,
    pub posts: Vec<SketchItem>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchItem {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    pub id: u64,
    pub comment_count: u32,
    pub user: SketchUser,
    pub is_r18: bool,
    pub media: Vec<SketchMedia>,
    pub tags: Vec<String>,
    pub text_fragments: Vec<SketchTextFragment>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchImpressions {
    pub feedbacks: Vec<SketchImpression>,
    pub item: SketchItem,
}

#[derive(Deserialize, Serialize)]
pub struct SketchImpression {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    pub id: u64,
    #[serde(rename = "type")]
    pub t: String,
    pub user: SketchUser,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchLives {
    pub lives: Vec<SketchLive>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchLive {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    pub id: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub user: SketchUser,
    pub name: String,
    pub description_fragments: Vec<SketchTextFragment>,
    pub is_r18: bool,
    pub is_broadcasting: bool,
    pub audience_count: u32,
    pub total_audience_count: u32,
    pub heart_count: u32,
    pub chat_count: u32,
}

#[derive(Deserialize, Serialize)]
pub struct SketchWall {
    pub items: Vec<SketchItem>,
}

#[derive(Deserialize, Serialize)]
pub struct SketchLiveOwner {
    pub hls_movie: SketchShortMedia,
}

#[derive(Deserialize, Serialize)]
pub struct SketchApiResponse<T> {
    pub data: T,
}

impl<T> SketchApiResponse<T> {}

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

// https://sketch.pixiv.net/api/walls/public.json
pub async fn fetch_public_wall(
    client: &awc::Client,
    since: Option<u64>,
    max: Option<u64>,
) -> Result<SketchApiResponse<SketchWall>, ApiError> {
    let url: Cow<str> = match (since, max) {
        (Some(since), None) => {
            format!("https://sketch.pixiv.net/api/walls/public.json?since_id={since}").into()
        }
        (None, Some(max)) => {
            format!("https://sketch.pixiv.net/api/walls/public.json?max_id={max}").into()
        }
        _ => "https://sketch.pixiv.net/api/walls/public.json".into(),
    };
    fetch_json(client, &url).await
}

// https://sketch.pixiv.net/api/walls/tags/%E3%83%9D%E3%82%B1%E3%83%A2%E3%83%B3.json
pub async fn fetch_tag_wall(
    client: &awc::Client,
    tag: &str,
) -> Result<SketchApiResponse<SketchWall>, ApiError> {
    let tag = percent_encoding::utf8_percent_encode(tag, percent_encoding::NON_ALPHANUMERIC);
    let url = format!("https://sketch.pixiv.net/api/walls/tags/{tag}.json");
    fetch_json(client, &url).await
}
