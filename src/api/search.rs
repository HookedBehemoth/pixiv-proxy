use std::str::FromStr;

use crate::get_param_or_num;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
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

impl std::str::FromStr for SearchOrder {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "date_d" => Ok(Self::DateDescending),
            "date" => Ok(Self::DateAscending),
            "popular_d" => Ok(Self::Popular),
            "popular_male_d" => Ok(Self::PopularMale),
            "popular_female_d" => Ok(Self::PopularFemale),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for SearchOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
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

impl std::str::FromStr for SearchRating {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "r18" => Ok(Self::Adult),
            "safe" => Ok(Self::Safe),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for SearchRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
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

impl std::str::FromStr for SearchMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s_tag_full" => Ok(Self::TagsPerfect),
            "s_tag" => Ok(Self::TagsPartial),
            "s_tc" => Ok(Self::TitleCaption),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct SearchRequest {
    pub page: u32,
    pub order: SearchOrder,
    pub rating: SearchRating,
    pub mode: SearchMode,
    // pub q: Option<String>,
}

impl From<&rouille::Request> for SearchRequest {
    fn from(req: &rouille::Request) -> Self {
        Self {
            page: get_param_or_num!(req, "p", 1),
            order: req
                .get_param("order")
                .and_then(|s| SearchOrder::from_str(&s).ok())
                .unwrap_or(SearchOrder::DateDescending),
            rating: req
                .get_param("mode")
                .and_then(|s| SearchRating::from_str(&s).ok())
                .unwrap_or(SearchRating::All),
            mode: req
                .get_param("s_mode")
                .and_then(|s| SearchMode::from_str(&s).ok())
                .unwrap_or(SearchMode::TagsPerfect),
        }
    }
}

// https://www.pixiv.net/ajax/search/artworks/世話やきキツネの仙狐さん?word=世話やきキツネの仙狐さん&order=date_d&mode=r18&p=3&s_mode=s_tag&type=all&lang=en
pub fn fetch_search(
    client: &ureq::Agent,
    tags: &str,
    query: &SearchRequest,
) -> Result<PixivSearch, ApiError> {
    let tags = percent_encoding::utf8_percent_encode(tags, percent_encoding::NON_ALPHANUMERIC);
    let url = format!("https://www.pixiv.net/ajax/search/artworks/{}?word={}&order={}&mode={}&p={}&s_mode={}&type=all&lang=en", &tags, &tags, query.order, query.rating, query.page, query.mode);

    fetch(client, &url)
}
