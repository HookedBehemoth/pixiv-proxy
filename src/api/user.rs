use std::collections::HashMap;

use super::{common::PixivSearchResult, error::ApiError, fetch::fetch};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PixivIllustrations {
    illusts: serde_json::Value,
    manga: serde_json::Value,
}

// https://www.pixiv.net/ajax/user/3384404/profile/all?lang=en
pub fn fetch_user_illust_ids(client: &ureq::Agent, user_id: u64) -> Result<Vec<u32>, ApiError> {
    let url = format!(
        "https://www.pixiv.net/ajax/user/{}/profile/all?lang=en",
        user_id
    );

    let ids: PixivIllustrations = fetch(client, &url)?;

    let works_iter = ids
        .illusts
        .as_object()
        .map(|m| m.into_iter().map(|(k, _)| k.parse::<u32>().unwrap()));
    let manga_iter = ids
        .manga
        .as_object()
        .map(|m| m.into_iter().map(|(k, _)| k.parse::<u32>().unwrap()));
    let mut ids = match (works_iter, manga_iter) {
        (Some(works), Some(manga)) => works.chain(manga).collect(),
        (Some(works), None) => works.collect(),
        (None, Some(manga)) => manga.collect(),
        (None, None) => vec![],
    };

    ids.sort_unstable();
    ids.reverse();

    Ok(ids)
}

#[derive(Deserialize)]
struct PixivWorks {
    pub works: HashMap<u32, PixivSearchResult>,
}

// https://www.pixiv.net/ajax/user/3384404/profile/illusts?ids[]=84485304&ids[]=84473597&work_category=illust&is_first_page=0&lang=en
pub fn fetch_user_illustrations(
    client: &ureq::Agent,
    user_id: u64,
    ids: &[u32],
) -> Result<Vec<PixivSearchResult>, ApiError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let url = format!("https://www.pixiv.net/ajax/user/{}/profile/illusts?{}&work_category=illust&is_first_page=0&lang=en", 
        user_id,
        ids.iter()
            .map(|id| format!("ids[]={}", id))
            .collect::<Vec<String>>()
            .join("&")
    );

    let elements: PixivWorks = fetch(client, &url)?;

    let mut elements: Vec<(u32, PixivSearchResult)> = elements.works.into_iter().collect();
    elements.sort_unstable_by_key(|s| s.0);
    elements.reverse();
    let elements = elements.into_iter().map(|(_, s)| s).collect();

    Ok(elements)
}

#[derive(Deserialize)]
pub struct PixivBookmarks {
    pub works: Vec<PixivSearchResult>,
    pub total: usize,
}

// https://www.pixiv.net/ajax/user/42433315/illusts/bookmarks?tag=&offset=0&limit=48&rest=show&lang=en
pub fn fetch_user_bookmarks(
    client: &ureq::Agent,
    user_id: u64,
    tag: &str,
    offset: u32,
    limit: u32,
) -> Result<PixivBookmarks, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/user/{}/illusts/bookmarks?tag={}&offset={}&limit={}&rest=show&lang=en", user_id, tag, offset, limit);

    println!("{}", url);

    fetch(client, &url)
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct PixivUser {
    pub name: String,
    pub image_big: String,
    pub comment_html: String,
}

// https://www.pixiv.net/ajax/user/38588185?full=1&lang=en
pub fn fetch_user_profile(client: &ureq::Agent, user_id: u64) -> Result<PixivUser, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/user/{}?full=1&lang=en", user_id);

    fetch(client, &url)
}

// https://www.pixiv.net/ajax/user/38588185/works/latest?lang=en
// https://www.pixiv.net/ajax/user/3384404/illusts/tag?tag=R-18&offset=0&limit=48&lang=en
// https://www.pixiv.net/ajax/user/3384404/profile/top?lang=en
