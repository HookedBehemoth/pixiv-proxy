use maud::html;

use crate::{
    api::{
        error::ApiError,
        search::{fetch_search, SearchRequest},
    },
    get_param_or_str,
    render::{datetime::DateTimeWrapper, document::document, nav::render_nav},
    util,
};

pub fn scroll(
    client: &ureq::Agent,
    query: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let qtype = get_param_or_str!(query, "qtype", "search");
    let words = get_param_or_str!(query, "q", "");
    let query = SearchRequest::from(query);
    let (data, total) = match &qtype[..] {
        "author" => {
            let user_id = words.parse::<u64>().unwrap();
            super::users::fetch_illustrations(client, user_id, query.page, &words, false)?
        }
        _ => {
            let search = fetch_search(client, &words, &query)?;
            (search.illust_manga.data, search.illust_manga.total)
        }
    };

    let document = document(
        &words,
        html! {
            h1 { (words) }
            p { (total) }
            ul.scroll.artworks {
                @for illust in data.iter() {
                    li {
                        h2 { a href=(format!("/artworks/{}", illust.id)) { (illust.title) } }

                        @if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&illust.update_date) {
                            p { (DateTimeWrapper(date.into())) }
                            @let img_base = format!(
                                "/imageproxy/img-master/img/{}/{}",
                                date.format("%Y/%m/%d/%H/%M/%S"),
                                illust.id
                            );
                            @let (width, height) = util::scale_by_aspect_ratio(illust.width, illust.height, 900, 900);
                            @match illust.illust_type {
                                2 => {
                                    @let thumbnail = format!("{}_master1200.jpg", img_base);
                                    @let video = format!("/ugoira/{}", illust.id);
                                    video src=(&video) poster=(&thumbnail) width=(width) height=(height) controls muted loop playsinline preload="none" {}
                                }
                                _ => {
                                    img src=(format!("{}_p0_master1200.jpg", img_base)) width=(width) height=(height) alt="" loading="lazy";
                                    @if illust.page_count > 1 {
                                        details {
                                            summary {
                                                (format!("{} more...", illust.page_count - 1))
                                            }
                                            @for i in 1..illust.page_count {
                                                img src=(format!("{}_p{}_master1200.jpg", img_base, i)) alt="" loading="lazy";
                                            }
                                        }
                                    }
                                }
                            }
                        } @else {
                            img src=(&illust.url) width="250" height="250" alt=(illust.id);
                        }
                    }
                }
            }
            @if total > data.len() {
                @let format = format!("scroll?qtype={qtype}&q={words}&mode={}&order={}&s_mode={}&p=", query.rating, query.order, query.mode);
                (render_nav(query.page, total, 60, &format))
            }
        },
        None,
    );

    Ok(rouille::Response::html(document.into_string()))
}
