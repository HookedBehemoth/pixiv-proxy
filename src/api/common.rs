use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiResponse<T> {
    pub error: bool,
    pub body: Option<T>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct PixivSearchResult {
    pub id: String,
    pub title: String,
    #[serde(rename = "xRestrict")]
    pub r18: u32,
    #[serde(rename = "pageCount")]
    pub page_count: u32,
    // #[serde(rename = "illustType")]
    // pub illust_type: u8,
    pub url: String,
}
