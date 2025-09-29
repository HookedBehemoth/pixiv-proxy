use maud::{html, PreEscaped};

use crate::{
    api::{artwork::{fetch_artwork, fetch_pages, Page}, error::ApiError},
    render::{datetime::DateTimeWrapper, document::document, svg},
    util,
};

const UGOIRA: u8 = 2;

pub fn artwork(client: &ureq::Agent, id: u64) -> Result<rouille::Response, ApiError> {
    let artwork = fetch_artwork(client, id)?;
    let is_ugoira = artwork.illust_type == UGOIRA;
    let pages = if !is_ugoira && artwork.page_count > 1 {
        fetch_pages(client, id)?
    } else {
        vec![Page{
            urls: artwork.urls.clone(),
            width: artwork.width,
            height: artwork.height,
        }]
    };

    let date = chrono::DateTime::parse_from_rfc3339(&artwork.create_date);

    let document = document(
        &artwork.illust_title,
        html! {
            /* Title */
            h1 { (&artwork.illust_title) }
            /* Author */
            @let link = format!("/users/{}", artwork.user_id);
            p.illust__author { a href=(&link) { (&artwork.user_name) } }
            /* Description */
            @if !artwork.description.is_empty() {
                p { (PreEscaped(&artwork.description)) }
            }
            /* Tags */
            (artwork.tags)
            /* Meta */
            p.illust__meta {
                @if date.is_ok() {
                    time datetime=(&artwork.create_date) {
                        (DateTimeWrapper(date.unwrap().into()))
                    }
                }
                (svg::like())
                (artwork.like_count)
                (svg::heart())
                (artwork.bookmark_count)
                (svg::eye())
                (artwork.view_count)
            }
            /* Images */
            div.artworks {
                @if is_ugoira && cfg!(feature = "ugoira") {
                    @let src = format!("/ugoira/{}", id);
                    video poster=(&artwork.urls.original) src=(&src) controls autoplay muted loop playsinline {}
                } @else {
                    @for page in &pages {
                        @let small = &page.urls.small;
                        @let regular = &page.urls.regular;
                        @let original = &page.urls.original;
                        @let aspect = page.width as f32 / page.height as f32;
                        @let (small_width, regular_width) = if aspect > 1.0 {
                            (540, 1200)
                        } else {
                            (
                                (540 as f32 * aspect).ceil() as u32,
                                (1200 as f32 * aspect).ceil() as u32
                            )
                        };

                        a href=(original) download {
                            picture {
                                source media=(format!("(max-width: {small_width}px)")) srcset=(small);
                                source media=(format!("(max-width: {regular_width}px)")) srcset=(regular);
                                img src=(original)
                                    width=(page.width) height=(page.height)
                                    alt=(&artwork.alt)
                                    loading="lazy";
                            }
                        }
                    }
                }
            }
            /* Comments */
            @if artwork.comment_count > 0 {
                div.comments_wrapper {
                    button endpoint=(format!("/comments/{}", id)) type="button" onclick="inject(this)" {
                        "Load Comments"
                    }
                }
            }
        },
        Some(html! {
            meta name="twitter:title" content=(&artwork.illust_title);
            meta name="twitter:creator" content=(&artwork.user_name);
            meta name="twitter:image" content=(artwork.urls.original);
            @if is_ugoira {
                meta name="twitter:card" content="player";
                @let url = format!("/ugoira/{}", id);
                meta name="twitter:player:stream" content=(&url);
                meta name="twitter:player:stream:content_type" content="video/mp4";
                meta name="twitter:player:width" content=(artwork.width);
                meta name="twitter:player:height" content=(artwork.height);
            } @else {
                @for page in &pages {
                    meta name="twitter:image" content=(&page.urls.original);
                }
                meta name="twitter:card" content="summary_large_image";
            }
            @let description = util::truncate(&artwork.description, 200);
            meta property="og:description" content=(&description);
            /* Insert javascript if needed */
            @if artwork.comment_count > 0 {
                script {
                    (PreEscaped(include_str!("../dynamic.js")))
                }
            }
        }),
    );

    Ok(rouille::Response::html(document.into_string()))
}
