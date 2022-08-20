use maud::{html, Render};

use crate::api::sketch::SketchItem;

impl Render for SketchItem {
    fn render(&self) -> maud::Markup {
        html! {
            @let user = &self.user;
            div.head {
                @let photo = &user.icon.photo.pxsq60;
                img src=(photo.url) alt="" width=(photo.width) height=(photo.height) loading="lazy";
                span { (user.name) }
                span { (self.created_at.to_string()) }
            }
            center {
                @for media in &self.media {
                    a href=(&media.photo.original.url) {
                        @let photo = &media.photo.pxw540;
                        img src=(photo.url) alt="" width=(photo.width) height=(photo.height) loading="lazy";
                    }
                }
            }
            div.text {
                @for text in &self.text_fragments {
                    @match text.t.as_str() {
                        "plain" => (text.body),
                        "url" => a href=(text.normalized_body) { (text.body) },
                        "tag" => a href=(format!("/sketch/tags/{}", percent_encoding::utf8_percent_encode(&text.normalized_body, percent_encoding::NON_ALPHANUMERIC))) { (text.body) },
                        t => (format!("unknown text type: {t}")),
                    }
                }
            }
        }
    }
}
