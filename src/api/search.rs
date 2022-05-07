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

// https://www.pixiv.net/en/tags/世話やきキツネの仙狐さん/artworks?mode=r18&p=3&s_mode=s_tag
pub fn fetch_search(
    client: &ureq::Agent,
    query: &str,
    order: &str,
    mode: &str,
    page: u32,
    search_mode: &str,
) -> Result<PixivSearch, ApiError> {
    // https://www.pixiv.net/ajax/search/artworks/世話やきキツネの仙狐さん?word=世話やきキツネの仙狐さん&order=date_d&mode=r18&p=3&s_mode=s_tag&type=all&lang=en
    // let url = format!("https://www.pixiv.net/ajax/search/artworks/{}", query);
    // let query = [
    //     ("word", query),
    //     ("order", order),
    //     ("mode", mode),
    //     ("p", page),
    //     ("s_mode", search_mode),
    //     ("type", "all"),
    //     ("lang", "en"),
    // ];
    let query = percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC);
    let url = format!("https://www.pixiv.net/ajax/search/artworks/{}?word={}&order={}&mode={}&p={}&s_mode={}&type=all&lang=en", &query, &query, order, mode, page, search_mode);

    fetch(client, &url)
}
