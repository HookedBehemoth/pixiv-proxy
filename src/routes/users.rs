use maud::{html, PreEscaped};

use crate::{
    api::{
        common::PixivSearchResult,
        error::ApiError,
        user::{
            fetch_user_bookmarks, fetch_user_illust_ids, fetch_user_illustrations,
            fetch_user_profile,
        },
    },
    get_param_or_num, get_param_or_str,
    render::{alt::render_alt_author, document::document, grid::render_grid, nav::render_nav},
};

pub fn artworks(
    client: &ureq::Agent,
    id: u64,
    query: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    user(client, id, query, false)
}

pub fn bookmarks(
    client: &ureq::Agent,
    id: u64,
    query: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    user(client, id, query, true)
}

fn user(
    client: &ureq::Agent,
    user_id: u64,
    query: &rouille::Request,
    bookmarks: bool,
) -> Result<rouille::Response, ApiError> {
    let page = get_param_or_num!(query, "p", 1);
    let query = get_param_or_str!(query, "q", "");
    let user = fetch_user_profile(client, user_id)?;
    let (elements, count) = fetch_illustrations(client, user_id, page, &query, bookmarks)?;

    let image = &user.image_big;

    let document = document(
        &user.name,
        html! {
            header.author {
                img.logo src=(&image) alt=(&user.name) width="170";
                h1 { (&user.name) }
                (render_alt_author(user_id, page))
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
                (render_nav(page, count, 60, &format))
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

    Ok(rouille::Response::html(document.into_string()))
}

pub fn fetch_illustrations(
    client: &ureq::Agent,
    user_id: u64,
    page: u32,
    tags: &str,
    bookmarks: bool,
) -> Result<(Vec<PixivSearchResult>, usize), ApiError> {
    if !bookmarks {
        let ids = fetch_user_illust_ids(client, user_id)?;

        let count = ids.len();
        let start = (page - 1) * 60;
        let end = std::cmp::min(start + 60, count as u32);
        let slice = &ids[start as usize..end as usize];

        let elements = fetch_user_illustrations(client, user_id, slice)?;

        Ok((elements, count))
    } else {
        let bookmarks = fetch_user_bookmarks(client, user_id, tags, (page - 1) * 60, 60)?;

        Ok((bookmarks.works, bookmarks.total))
    }
}
