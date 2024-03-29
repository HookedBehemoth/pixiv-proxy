use super::de::{deserialize_number_unconditionally, strip_url_prefix};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiResponse<T> {
    pub error: bool,
    pub body: Option<T>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct PixivSearchResult {
    #[serde(deserialize_with = "deserialize_number_unconditionally")]
    pub id: u64,
    pub title: String,
    #[serde(
        rename = "userId",
        deserialize_with = "deserialize_number_unconditionally"
    )]
    pub user_id: u64,
    pub user_name: String,
    /* Note: This appears to always be empty */
    pub description: String,
    pub update_date: String,
    pub create_date: String,
    #[serde(rename = "xRestrict")]
    pub r18: u32,
    pub page_count: u32,
    pub illust_type: u8,
    #[serde(deserialize_with = "strip_url_prefix")]
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub is_masked: bool,
}
