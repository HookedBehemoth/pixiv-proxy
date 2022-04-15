use super::{error::ApiError, fetch::fetch};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pixiv {
}

// 
pub fn fetch_(
    client: &ureq::Agent, 
) -> Result<PixivSearch, ApiError> {
    let url = format!("https://www.pixiv.net/ajax/search/artworks/{}?word={}&order={}&mode={}&p={}&s_mode={}&type=all&lang=en", query, query, order, mode, page, search_mode);

    fetch(client, &url)
}
