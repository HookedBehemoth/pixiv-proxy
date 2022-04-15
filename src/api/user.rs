use std::collections::HashMap;

use super::{common::PixivSearchResult, error::ApiError, fetch::fetch};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PixivIllustrations {
    illusts: HashMap<u32, ()>,
}

// https://www.pixiv.net/ajax/user/3384404/profile/all?lang=en
pub fn fetch_user_illust_ids(client: &ureq::Agent, user_id: &str) -> Result<Vec<u32>, ApiError> {
    let url = format!(
        "https://www.pixiv.net/ajax/user/{}/profile/all?lang=en",
        user_id
    );

    let ids: PixivIllustrations = fetch(client, &url)?;

    let mut ids: Vec<u32> = ids.illusts.into_iter().map(|(k, _)| k).collect();

    ids.sort_unstable();
    ids.reverse();

    Ok(ids)
}

#[derive(Deserialize)]
pub struct PixivWorks {
    pub works: HashMap<u32, PixivSearchResult>,
}

// https://www.pixiv.net/ajax/user/3384404/profile/illusts?ids[]=84485304&ids[]=84473597&work_category=illust&is_first_page=0&lang=en
pub fn fetch_user_illustrations(
    client: &ureq::Agent,
    user_id: &str,
    ids: &[u32],
) -> Result<PixivWorks, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/user/{}/profile/illusts?{}&work_category=illust&is_first_page=0&lang=en", 
        user_id,
        ids.into_iter()
            .map(|id| format!("ids[]={}", id))
            .collect::<Vec<String>>()
            .join("&")
    );

    fetch(client, &url)
}

#[derive(Deserialize)]
pub struct PixivUser {
    pub name: String,
    #[serde(rename = "imageBig")]
    pub image_big: String,
    #[serde(rename = "commentHtml")]
    pub comment_html: String,
}

// https://www.pixiv.net/ajax/user/38588185?full=1&lang=en
pub fn fetch_user_profile(client: &ureq::Agent, user_id: &str) -> Result<PixivUser, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/user/{}?full=1&lang=en", user_id);

    fetch(client, &url)
}

// https://www.pixiv.net/ajax/user/38588185/works/latest?lang=en
// https://www.pixiv.net/ajax/user/3384404/illusts/tag?tag=R-18&offset=0&limit=48&lang=en
// https://www.pixiv.net/ajax/user/3384404/profile/top?lang=en
