use std::{fmt, str::FromStr};

use super::{error::ApiError, fetch::fetch_json};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Deserialize)]
pub struct Ranking {
    pub contents: Vec<RankingItem>,
    pub date: String,
    #[serde(deserialize_with = "string_or_none")]
    pub prev_date: Option<String>,
    #[serde(deserialize_with = "string_or_none")]
    pub next_date: Option<String>,
    pub rank_total: usize,
}

fn string_or_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNone(Option<String>);

    impl<'de> Visitor<'de> for StringOrNone {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("date or false")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<String>, E>
        where
            E: de::Error,
        {
            Ok(Some(FromStr::from_str(value).unwrap()))
        }

        fn visit_bool<E>(self, _: bool) -> Result<Option<String>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrNone(None))
}

#[derive(Deserialize)]
pub struct RankingItem {
    pub title: String,
    pub url: String,
    pub illust_id: u32,
    pub width: u32,
    pub height: u32,
    pub illust_page_count: String,
    pub illust_upload_timestamp: u64,
}

pub async fn fetch_ranking(
    client: &awc::Client,
    date: Option<&String>,
    page: u32,
) -> Result<Ranking, ApiError> {
    let date = date.map(|d| format!("&date={}", d));
    let url = format!(
        "https://www.pixiv.net/ranking.php?mode=daily&p={}&format=json{}",
        page,
        date.unwrap_or_else(|| "".to_string())
    );

    fetch_json::<Ranking>(client, &url).await
}
