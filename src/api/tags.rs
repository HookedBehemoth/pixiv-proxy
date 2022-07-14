use serde::Deserialize;

#[derive(Deserialize)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

#[derive(Deserialize)]
pub struct Tag {
    pub tag: String,
    pub translation: Option<Translation>,
}

#[derive(Deserialize)]
pub struct Translation {
    pub en: Option<String>,
}
