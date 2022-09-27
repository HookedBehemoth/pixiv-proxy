use actix_web::{web, Result};
use maud::{html, Markup, PreEscaped};

use crate::{
    api::artwork::fetch_artwork,
    render::{datetime::DateTimeWrapper, document::document, svg},
    util,
};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    web::resource(["/en/artworks/{id}", "/artworks/{id}"]).route(web::get().to(artwork))
}

async fn artwork(client: web::Data<awc::Client>, id: web::Path<u64>) -> Result<Markup> {
    let id = id.into_inner();
    let artwork = fetch_artwork(&client, id).await?;

    let image = &artwork.urls.original;
    let date = chrono::DateTime::parse_from_rfc3339(&artwork.create_date);

    let docs = document(
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
                @match artwork.illust_type {
                    2 => {
                        @if cfg!(feature = "ugoira") {
                            @let src = format!("/ugoira/{}", id);
                            video poster=(&image) src=(&src) controls autoplay muted loop playsinline {}
                        } @else {
                            img src=(&image) alt="";
                        }
                    },
                    _ => @for url in std::iter::once(image.clone())
                        .chain(
                            (1..artwork.page_count).map(|i|
                                image.clone().replace("_p0.", &format!("_p{}.", i))
                            )
                        ) {
                        img src=(&url) alt=(&artwork.alt);
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
            meta name="twitter:image" content=(image);
            @match artwork.illust_type {
                2 => {
                    meta name="twitter:card" content="player";
                    @let url = format!("/ugoira/{}", id);
                    meta name="twitter:player:stream" content=(&url);
                    meta name="twitter:player:stream:content_type" content="video/mp4";
                    meta name="twitter:player:width" content=(artwork.width);
                    meta name="twitter:player:height" content=(artwork.height);
                },
                _ => {
                    @for url in std::iter::once(image.clone())
                        .chain(
                            (1..artwork.page_count).map(|i|
                                image.clone().replace("_p0.", &format!("_p{}.", i))
                            )
                        ) {
                        meta name="og:image" src=(&url);
                    }
                    meta name="twitter:card" content="summary_large_image";
                }
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

    Ok(docs)
}
