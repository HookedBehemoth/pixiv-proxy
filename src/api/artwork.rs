use super::{error::ApiError, fetch::fetch, tags::Tags};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PixivUrls {
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
    pub user_id: String,
    pub description: String,
    pub create_date: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
    pub urls: PixivUrls,
    pub tags: Tags,
    pub comment_count: u32,
}

pub fn fetch_artwork(client: &ureq::Agent, id: &str) -> Result<Artwork, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/illust/{}", id);

    fetch(client, &url)
}
