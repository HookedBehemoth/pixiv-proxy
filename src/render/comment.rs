use crate::api::comments::PixivComment;
use ::phf::{phf_map, Map};
use maud::{html, Markup, Render};

impl Render for &PixivComment {
    fn render(&self) -> Markup {
        html! {
            li {
                @let user = format!("/users/{}", self.user_id);
                div.pfp {
                    a href=(&user) {
                        img src=(&self.img) width="40" loading="lazy";
                    }
                }
                div.comment {
                    h3 { a href=(&user) { (&self.user_name) } }
                    @if let Some(stamp) = &self.stamp_id {
                        @let stamp_url = format!("/stamp/{}", stamp);
                        img.content src=(&stamp_url) width="80" height="80" loading="lazy";
                    } @else {
                        (render_comment_text(&self.comment))
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

fn render_comment_text(comment: &str) -> Markup {
    let chars = comment.chars();
    let mut in_stamp = false;
    let mut sequences = vec![];
    let mut emoji = String::new();
    let mut seq = String::new();
    for char in chars {
        match (in_stamp, char) {
            (_, '(') => {
                if in_stamp {
                    seq.push('(');
                    seq.push_str(&emoji);
                    emoji = String::new();
                }
                in_stamp = true;
                /* Flush pending comment */
                if !seq.is_empty() {
                    sequences.push(html! { (seq) });
                    seq = String::new()
                }
            }
            (false, c) => seq.push(c),
            (true, c) => {
                if char.is_alphanumeric() {
                    emoji.push(c);
                } else if c == ')' {
                    if let Some(stamp) = lookup_emoji(&emoji) {
                        sequences.push(html! {
                            img.emoji src=(stamp) alt=(emoji) width="24" height="24" {}
                        });
                    } else {
                        seq.push('(');
                        seq.push_str(&emoji);
                        seq.push(')');
                    }
                    emoji = String::new();
                    in_stamp = false;
                } else {
                    seq.push('(');
                    seq.push_str(&emoji);
                    seq.push(c);
                    emoji = String::new();
                    in_stamp = false;
                }
            }
        }
    }

    let markup = html! {
        p.content {
            @for seq in sequences {
                (seq)
            }
            @if in_stamp && !emoji.is_empty() {
                "(" (emoji)
            } @else if !seq.is_empty() {
                (seq)
            }
        }
    };

    markup
}

fn lookup_emoji(name: &str) -> Option<String> {
    EMOJI_LOOKUP
        .get(name)
        .map(|id| format!("/simg/common/images/emoji/{id}.png"))
}

const EMOJI_LOOKUP: Map<&str, u16> = phf_map! {
    "normal" => 101,
    "surprise" => 102,
    "serious" => 103,
    "heaven" => 104,
    "happy" => 105,
    "excited" => 106,
    "sing" => 107,
    "cry" => 108,
    "normal2" => 201,
    "shame2" => 202,
    "love2" => 203,
    "interesting2" => 204,
    "blush2" => 205,
    "fire2" => 206,
    "angry2" => 207,
    "shine2" => 208,
    "panic2" => 209,
    "normal3" => 301,
    "satisfaction3" => 302,
    "surprise3" => 303,
    "smile3" => 304,
    "shock3" => 305,
    "gaze3" => 306,
    "wink3" => 307,
    "happy3" => 308,
    "excited3" => 309,
    "love3" => 310,
    "normal4" => 401,
    "surprise4" => 402,
    "serious4" => 403,
    "love4" => 404,
    "shine4" => 405,
    "sweat4" => 406,
    "shame4" => 407,
    "sleep4" => 408,
    "heart" => 501,
    "teardrop" => 502,
    "star" => 503,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_comment() {
        let comment = "Basic Comment without any emoji";
        let markup = render_comment_text(comment);
        assert_eq!(
            markup.into_string(),
            r#"<p class="content">Basic Comment without any emoji</p>"#
        );
    }

    #[test]
    fn simple_emoji() {
        let comment = "Basic Comment with (normal) emoji";
        let markup = render_comment_text(comment);
        assert_eq!(
            markup.into_string(),
            r#"<p class="content">Basic Comment with <img class="emoji" src="/simg/common/images/emoji/101.png" alt="normal" width="24" height="24"></img> emoji</p>"#
        );
    }

    #[test]
    fn end_emoji() {
        let comment = "Basic comment with the emoji at the end (love3)";
        let markup = render_comment_text(comment);
        assert_eq!(
            markup.into_string(),
            r#"<p class="content">Basic comment with the emoji at the end <img class="emoji" src="/simg/common/images/emoji/310.png" alt="love3" width="24" height="24"></img></p>"#
        );
    }

    #[test]
    fn unterminated_emoji() {
        let comment = "Unterminated (normal";
        let markup = render_comment_text(comment);
        assert_eq!(
            markup.into_string(),
            r#"<p class="content">Unterminated (normal</p>"#
        )
    }

    #[test]
    fn emoji_nested_simple() {
        let comment = "Emoji ((normal)) nested";
        let markup = render_comment_text(comment);
        assert_eq!(
            markup.into_string(),
            r#"<p class="content">Emoji (<img class="emoji" src="/simg/common/images/emoji/101.png" alt="normal" width="24" height="24"></img>) nested</p>"#
        );
    }

    #[test]
    fn emoji_nested() {
        let comment = "Emoji (in colons (normal) nested)";
        let markup = render_comment_text(comment);
        assert_eq!(
            markup.into_string(),
            r#"<p class="content">Emoji (in colons <img class="emoji" src="/simg/common/images/emoji/101.png" alt="normal" width="24" height="24"></img> nested)</p>"#
        );
    }

    #[test]
    fn non_existant_stamp() {
        let comment = "Emoji that is (nonexistant)";
        let markup = render_comment_text(comment);
        assert_eq!(
            markup.into_string(),
            r#"<p class="content">Emoji that is (nonexistant)</p>"#
        );
    }
}
