use maud::html;

use crate::{
    api::{
        error::ApiError,
        search::{fetch_search, SearchRequest},
    },
    render::{
        alt::render_alt_search, document::document, grid::render_grid, nav::render_nav,
        search::render_options,
    },
    settings::get_blocked_userids,
};

pub fn tags(
    client: &ureq::Agent,
    tags: &str,
    request: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    render_search(client, tags, request)
}

pub fn query_search(
    client: &ureq::Agent,
    request: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let words = request
        .get_param("q")
        .ok_or_else(|| ApiError::External(403, "No query".into()))?;
    render_search(client, &words, request)
}

fn render_search(
    client: &ureq::Agent,
    tags: &str,
    request: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let blocked_set = get_blocked_userids(request);

    let query = SearchRequest::from(request);
    let search = fetch_search(client, tags, &query)?;

    let document = document(
        tags,
        html! {
            h1 { (&tags) }
            (&search.illust_manga.total)
            (render_alt_search(tags, &query))
            (render_options(tags, query.rating, query.order, query.mode))
            (render_grid(&search.illust_manga.data, &blocked_set))
            @if search.illust_manga.total > 60 {
                @let format = format!("/search?q={}&order={}&mode={}&s_mode={}&p=", tags, query.order, query.rating, query.mode);
                (render_nav(query.page, search.illust_manga.total, 60, &format))
            }
            p {
                "You have blocked " (blocked_set.len()) " Users. Some entries might be hidden."
            }
        },
        None,
    );

    Ok(rouille::Response::html(document.into_string()))
}
