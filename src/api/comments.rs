use super::{error::ApiError, fetch::fetch};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct PixivComments {
    pub comments: Vec<PixivComment>,
    pub has_next: bool,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct PixivComment {
    pub user_id: String,
    pub user_name: String,
    pub img: String,
    pub id: String,
    pub comment: String,
    pub stamp_id: Option<String>,
    pub comment_date: String,
    pub has_replies: Option<bool>,
}

// https://www.pixiv.net/ajax/illusts/comments/roots?illust_id=97276742&offset=0&limit=3&lang=en
pub fn fetch_comments(
    client: &ureq::Agent,
    id: u32,
    offset: u32,
    limit: u32,
) -> Result<PixivComments, ApiError> {
    let url = format!(
        "https://www.pixiv.net/ajax/illusts/comments/roots?illust_id={}&offset={}&limit={}&lang=en",
        id, offset, limit
    );

    fetch(client, &url)
}

// https://www.pixiv.net/ajax/illusts/comments/replies?comment_id=137840290&page=1&lang=en
pub fn fetch_replies(
    client: &ureq::Agent,
    id: u32,
    page: u32,
) -> Result<PixivComments, ApiError> {
    let url = format!(
        "https://www.pixiv.net/ajax/illusts/comments/replies?comment_id={}&page={}&lang=en",
        id, page
    );

    fetch(client, &url)
}
