use serde::Deserialize;

use super::{error::ApiError, fetch::fetch};

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct UgoiraMeta {
    pub src: String,
    pub original_src: String,
    pub frames: Vec<UgoiraFrame>,
}

#[repr(C)]
#[derive(Deserialize)]
pub struct UgoiraFrame {
    pub delay: u16,
}

pub fn fetch_ugoira_meta(client: &ureq::Agent, id: u32) -> Result<UgoiraMeta, ApiError> {
    let url = format!(
        "https://www.pixiv.net/ajax/illust/{}/ugoira_meta?lang=en",
        id
    );

    fetch(client, &url)
}
