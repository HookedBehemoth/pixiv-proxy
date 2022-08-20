use super::{
    de::{deserialize_number_unconditionally, strip_url_prefix},
    error::ApiError,
    fetch::fetch,
    tags::Tags,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PixivUrls {
    #[serde(deserialize_with = "strip_url_prefix")]
    pub original: String,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct Artwork {
    pub illust_type: u8,
    pub illust_title: String,
    pub page_count: u32,
    pub like_count: u32,
    pub bookmark_count: u32,
    pub view_count: u32,
    pub user_name: String,
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    pub user_id: u64,
    pub description: String,
    pub create_date: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
    pub urls: PixivUrls,
    pub tags: Tags,
    pub comment_count: u32,
}

pub async fn fetch_artwork(client: &awc::Client, id: u64) -> Result<Artwork, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/illust/{}", id);

    fetch(client, &url).await
}
