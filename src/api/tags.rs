use serde::Deserialize;

#[derive(Deserialize)]
pub struct Tags {
    tags: Vec<Tag>,
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

impl maud::Render for Tags {
    fn render(&self) -> maud::Markup {
        maud::html!(
            ul class="tags" {
                @for tag in self.tags.iter() {
                    @let link = format!("/tags/{}/artworks", tag.tag);
                    li class="tags__item" {
                        a href=(&link) { (&tag.tag) }
                        @if let Some(ref translation) = tag.translation {
                            @if let Some(ref en) = translation.en {
                                span class="tags__translation" { (&en) }
                            }
                        }
                    }
                }
            }
        )
    }
}
