use serde::Deserialize;

use super::{error::ApiError, fetch::fetch};

#[derive(Deserialize)]
pub struct UgoiraMeta {
    pub src: String,
    #[serde(rename = "originalSrc")]
    pub original_src: String,
    pub mime_type: UgoiraMimeType,
    pub frames: Vec<UgoiraFrame>,
}

#[derive(Deserialize)]
pub struct UgoiraFrame {
    pub delay: u16,
}

pub struct UgoiraMimeType {
    pub format: image::ImageFormat,
}

impl<'de> serde::Deserialize<'de> for UgoiraMimeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let format = match s.as_str() {
            "image/jpeg" => image::ImageFormat::Jpeg,
            "image/png" => image::ImageFormat::Png,
            _ => return Err(serde::de::Error::custom("invalid mime type")),
        };
        Ok(UgoiraMimeType { format })
    }
}

pub fn fetch_ugoira_meta(client: &ureq::Agent, id: u32) -> Result<UgoiraMeta, ApiError> {
    let url = format!(
        "https://www.pixiv.net/ajax/illust/{}/ugoira_meta?lang=en",
        id
    );

    fetch(client, &url)
}
