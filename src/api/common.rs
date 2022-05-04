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
    pub id: String,
    pub title: String,
    pub user_name: String,
    /* Note: This appears to always be empty */
    pub description: String,
    pub update_date: String,
    pub create_date: String,
    #[serde(rename = "xRestrict")]
    pub r18: u32,
    pub page_count: u32,
    pub illust_type: u8,
    pub url: String,
}
