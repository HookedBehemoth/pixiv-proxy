use actix_web::{get, web, Result};
use maud::{html, Markup};
use serde::Deserialize;

use crate::{
    api::search::{fetch_search, SearchRequest, SearchRating, SearchMode},
    render::{datetime::DateTimeWrapper, document::document, nav::render_nav},
    util,
};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    scroll
}

fn page_default() -> u32 {
    1
}

#[derive(Deserialize)]
pub struct RssRequest {
    #[serde(rename = "p", default = "page_default")]
    pub page: u32,
    qtype: String,
    #[serde(rename = "q")]
    words: String,
    #[serde(rename = "mode", default)]
    rating: SearchRating,
    #[serde(rename = "s_mode", default)]
    mode: SearchMode,
}

#[get("/scroll")]
async fn scroll(
    client: web::Data<awc::Client>,
    query: web::Query<SearchRequest>,
) -> Result<Markup> {
    let tags = query.q.as_ref().unwrap();
    let content = fetch_search(&client, tags, &query).await?;

    let doc = document(
        tags,
        html! {
            h1 { (tags) }
            p { (content.illust_manga.total) }
            ul.scroll.artworks {
                @for illust in content.illust_manga.data.iter() {
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
            @if content.illust_manga.total > content.illust_manga.data.len() {
                @let format = format!("scroll?q={}&mode={}&order={}&s_mode={}&p=", tags, query.rating, query.order, query.mode);
                (render_nav(query.page, content.illust_manga.total, 60, &format))
            }
        },
        None,
    );

    Ok(doc)
}
