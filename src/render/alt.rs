use maud::{html, Markup};

use crate::{api::search::SearchRequest, render::svg};

fn render_alt(options: &str, page: u32) -> Markup {
    html! {
        a href=(format!("/scroll{options}&p={page}", page = page)) { "Scroll..." }
        a href=(format!("/rss{}", options)) {
            (svg::rss())
            "RSS"
        }
    }
}

pub fn render_alt_author(id: u64, page: u32) -> Markup {
    let options = format!("?qtype=author&q={id}");
    render_alt(&options, page)
}

pub fn render_alt_search(q: &str, query: &SearchRequest) -> Markup {
    let options = format!("?qtype=searchq={q}&mode={rating}&order={order}&s_mode={mode}",
        rating = query.rating,
        order = query.order,
        mode = query.mode);
    render_alt(&options, query.page)
}