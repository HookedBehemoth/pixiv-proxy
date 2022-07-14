use super::{common::PixivSearchResult, error::ApiError, fetch::fetch};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PixivArray<T> {
    pub data: Vec<T>,
    pub total: usize,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct PixivSearch {
    pub illust_manga: PixivArray<PixivSearchResult>,
    // pub related_tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum SearchOrder {
    #[serde(rename = "date_d")]
    DateDescending,
    #[serde(rename = "date")]
    DateAscending,
    #[serde(rename = "popular_d")]
    Popular,
    #[serde(rename = "popular_male_d")]
    PopularMale,
    #[serde(rename = "popular_female_d")]
    PopularFemale,
}

impl SearchOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DateDescending => "date_d",
            Self::DateAscending => "date",
            Self::Popular => "popular_d",
            Self::PopularMale => "popular_male_d",
            Self::PopularFemale => "popular_female_d",
        }
    }
}

impl std::fmt::Display for SearchOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for SearchOrder {
    fn default() -> Self {
        SearchOrder::DateDescending
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum SearchRating {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "r18")]
    Adult,
    #[serde(rename = "safe")]
    Safe,
}

impl SearchRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Adult => "r18",
            Self::Safe => "safe",
        }
    }
}

impl std::fmt::Display for SearchRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for SearchRating {
    fn default() -> Self {
        SearchRating::All
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum SearchMode {
    #[serde(rename = "s_tag_full")]
    TagsPerfect,
    #[serde(rename = "s_tag")]
    TagsPartial,
    #[serde(rename = "s_tc")]
    TitleCaption,
}

impl SearchMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TagsPerfect => "s_tag_full",
            Self::TagsPartial => "s_tag",
            Self::TitleCaption => "s_tc",
        }
    }
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::TagsPerfect
    }
}

fn page_default() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    #[serde(rename = "p", default = "page_default")]
    pub page: u32,
    #[serde(default)]
    pub order: SearchOrder,
    #[serde(rename = "mode", default)]
    pub rating: SearchRating,
    #[serde(rename = "s_mode", default)]
    pub mode: SearchMode,
    pub q: Option<String>,
}

// https://www.pixiv.net/ajax/search/artworks/世話やきキツネの仙狐さん?word=世話やきキツネの仙狐さん&order=date_d&mode=r18&p=3&s_mode=s_tag&type=all&lang=en
pub async fn fetch_search(
    client: &awc::Client,
    tags: &str,
    query: &SearchRequest,
) -> Result<PixivSearch, ApiError> {
    let tags = percent_encoding::utf8_percent_encode(tags, percent_encoding::NON_ALPHANUMERIC);
    let url = format!("https://www.pixiv.net/ajax/search/artworks/{}?word={}&order={}&mode={}&p={}&s_mode={}&type=all&lang=en", &tags, &tags, query.order, query.rating, query.page, query.mode);

    fetch(client, &url).await
}
