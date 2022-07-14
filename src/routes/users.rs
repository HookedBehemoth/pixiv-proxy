use actix_web::{web, Result};
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use tokio::try_join;

use crate::{
    api::{
        common::PixivSearchResult,
        error::ApiError,
        user::{
            fetch_user_bookmarks, fetch_user_illust_ids, fetch_user_illustrations,
            fetch_user_profile,
        },
    },
    render::{document::document, grid::render_grid, nav::render_nav},
    routes::imageproxy::image_to_proxy,
};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    vec![
        web::resource([
            "/en/users/{id}",
            "/users/{id}",
            "/en/users/{id}/artworks",
            "/users/{id}/artworks",
        ])
        .route(web::get().to(artworks)),
        web::resource([
            "/en/users/{id}/bookmarks/artworks",
            "/users/{id}/bookmarks/artworks",
        ])
        .route(web::get().to(bookmarks)),
    ]
}

fn page_default() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
struct UserRequest {
    #[serde(rename = "p", default = "page_default")]
    page: u32,
    #[serde(rename = "q", default)]
    tag: String,
}

async fn artworks(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
    query: web::Query<UserRequest>,
) -> Result<Markup> {
    user(&client, *id, &query, false).await
}

async fn bookmarks(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
    query: web::Query<UserRequest>,
) -> Result<Markup> {
    user(&client, *id, &query, true).await
}

async fn user(
    client: &awc::Client,
    user_id: u64,
    query: &UserRequest,
    bookmarks: bool,
) -> Result<Markup> {
    let (user, (elements, count)) = try_join!(
        fetch_user_profile(client, user_id),
        fetch_illustrations(client, user_id, query, bookmarks),
    )?;

    let image = image_to_proxy(&user.image_big);

    let doc = document(
        &user.name,
        html! {
            header.author {
                img.logo src=(&image) alt=(&user.name) width="170";
                h1 { (&user.name) }
                @if !user.comment_html.is_empty() {
                    p { (PreEscaped(&user.comment_html)) }
                }
            }
            div.category {
                @if !bookmarks {
                    div { "Artworks" }
                    a href=(format!("/users/{}/bookmarks/artworks", user_id)) { "Bookmarks" }
                } @else {
                    a href=(format!("/users/{}", user_id)) { "Artworks" }
                    div { "Bookmarks" }
                }
            }
            div {
                (render_grid(&elements))
            }
            @if count > 60 {
                @let format = if !bookmarks {
                    format!("/users/{}?p=", user_id)
                } else {
                    format!("/users/{}/bookmarks/artworks?p=", user_id)
                };
                (render_nav(query.page, count, 60, &format))
            }
        },
        Some(html! {
            meta property="og:title" content=(&user.name);
            meta property="og:type" content="image";
            @let description = format!("{} Images", count);
            meta property="og:description" content=(&description);
            meta property="og:url" content=(&format!("/users/{}", user_id));
            meta property="og:image" content=(&image);
            meta property="og:image:width" content="170";
            meta property="og:image:height" content="170";
        }),
    );

    Ok(doc)
}

async fn fetch_illustrations(
    client: &awc::Client,
    user_id: u64,
    query: &UserRequest,
    bookmarks: bool,
) -> Result<(Vec<PixivSearchResult>, usize), ApiError> {
    if !bookmarks {
        let ids = fetch_user_illust_ids(client, user_id).await?;

        let count = ids.len();
        let start = (query.page - 1) * 60;
        let end = std::cmp::min(start + 60, count as u32);
        let slice = &ids[start as usize..end as usize];

        let elements = fetch_user_illustrations(client, user_id, slice).await?;

        Ok((elements, count))
    } else {
        let bookmarks =
            fetch_user_bookmarks(client, user_id, &query.tag, (query.page - 1) * 60, 60).await?;

        Ok((bookmarks.works, bookmarks.total))
    }
}
