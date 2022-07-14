use actix_web::{get, web, Result};
use maud::html;

use crate::{
    api::search::{fetch_search, SearchRequest},
    render::{document::document, grid::render_grid, nav::render_nav, search::render_options},
};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    (
        web::resource([
            "/tags/{tags}",
            "/en/tags/{tags}",
            "/tags/{tags}/artworks",
            "/en/tags/{tags}/artworks",
        ])
        .route(web::get().to(query_tags)),
        query_search,
    )
}

async fn query_tags(
    client: web::Data<awc::Client>,
    tags: web::Path<String>,
    query: web::Query<SearchRequest>,
) -> Result<maud::Markup> {
    render_search(&client, &tags, &query).await
}

#[get("/search")]
async fn query_search(
    client: web::Data<awc::Client>,
    query: web::Query<SearchRequest>,
) -> Result<maud::Markup> {
    render_search(&client, query.q.as_ref().unwrap(), &query).await
}

async fn render_search(
    client: &awc::Client,
    tags: &str,
    query: &SearchRequest,
) -> Result<maud::Markup> {
    let search = fetch_search(client, tags, query).await?;

    Ok(document(
        tags,
        html! {
            h1 { (&tags) }
            (&search.illust_manga.total)
            @let options = format!("?q={}&mode={}&order={}&s_mode={}", tags,
            query.rating,
            query.mode,
            query.mode);
            a href=(format!("/scroll{}&p={}", options, query.page)) { "Scroll..." }
            a href=(format!("/rss{}", options)) {
                svg width="20" height="20" viewBox="0 0 20 20" style="background-color:#f78422" {
                    circle fill="#fff" cx="4" cy="16" r="2" {}
                    g fill="none" stroke="#fff" stroke-width="3" {
                        path d="M2,4a14,14,0,0,1,14,14" {}
                        path d="M2,9a9,9,0,0,1,9,9" {}
                    }
                }
                "RSS"
            }
            (render_options(tags, query.rating, query.order, query.mode))
            (render_grid(&search.illust_manga.data))
            @if search.illust_manga.total > 60 {
                @let format = format!("/search?q={}&order={}&mode={}&s_mode={}&p=", tags, query.order, query.rating, query.mode);
                (render_nav(query.page, search.illust_manga.total, 60, &format))
            }
        },
        None,
    ))
}
