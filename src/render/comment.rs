use crate::api::comments::PixivComment;
use crate::util;
use maud::{html, Markup, Render};

impl Render for &PixivComment {
    fn render(&self) -> Markup {
        html! {
            li {
                @let user = format!("/users/{}", self.user_id);
                @let img = util::image_to_proxy(&self.img);
                div.pfp {
                    a href=(&user) {
                        img src=(&img) width="40" loading="lazy";
                    }
                }
                div.comment {
                    h3 { a href=(&user) { (&self.user_name) } }
                    @if let Some(stamp) = &self.stamp_id {
                        @let stamp_url = format!("/stamp/{}", stamp);
                        img.content src=(&stamp_url) width="80" height="80" loading="lazy";
                    } @else {
                        p.content { (&self.comment) }
                    }
                    p.date { (&self.comment_date) }
                    @if self.has_replies.unwrap_or(false) {
                        div.replies {
                            button endpoint=(format!("/replies/{}", self.id)) onclick="inject(this)" {
                                "Load replies"
                            }
                        }
                    }
                }
            }
        }
    }
}
