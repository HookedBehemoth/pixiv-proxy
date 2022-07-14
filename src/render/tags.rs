use crate::api::tags::Tags;

impl maud::Render for Tags {
    fn render(&self) -> maud::Markup {
        maud::html!(
            ul class="tags" {
                @for tag in self.tags.iter() {
                    @let link = format!("/tags/{}/artworks", percent_encoding::utf8_percent_encode(&tag.tag, percent_encoding::NON_ALPHANUMERIC));
                    li {
                        a href=(&link) { (&tag.tag) }
                        @if let Some(ref translation) = tag.translation {
                            @if let Some(ref en) = translation.en {
                                span { (&en) }
                            }
                        }
                    }
                }
            }
        )
    }
}
