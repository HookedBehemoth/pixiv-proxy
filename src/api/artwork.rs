use super::{error::ApiError, fetch::fetch, tags::Tags};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PixivUrls {
    pub original: String,
}

#[derive(Deserialize)]
pub struct Artwork {
    #[serde(rename = "illustType")]
    pub illust_type: u8,
    #[serde(rename = "illustTitle")]
    pub illust_title: String,
    #[serde(rename = "pageCount")]
    pub page_count: u32,
    #[serde(rename = "likeCount")]
    pub like_count: u32,
    #[serde(rename = "bookmarkCount")]
    pub bookmark_count: u32,
    #[serde(rename = "viewCount")]
    pub view_count: u32,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub description: String,
    #[serde(rename = "createDate")]
    pub create_date: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
    pub urls: PixivUrls,
    pub tags: Tags,
}

pub fn fetch_artwork(client: &ureq::Agent, id: &str) -> Result<Artwork, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/illust/{}", id);

    fetch(client, &url)
}
