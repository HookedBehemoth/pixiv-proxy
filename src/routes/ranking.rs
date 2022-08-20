use actix_web::{web, Result};
use maud::html;
use serde::Deserialize;

use crate::{
    api::{
        ranking::fetch_ranking,
        search::{SearchMode, SearchOrder, SearchRating},
    },
    render::{document::document, nav::render_nav, search::render_options},
    util,
};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    web::resource(["/", "/en"]).route(web::get().to(ranking))
}

fn page_default() -> u32 {
    1
}

#[derive(Deserialize)]
struct RankingRequest {
    #[serde(rename = "p", default = "page_default")]
    page: u32,
    date: Option<String>,
}

async fn ranking(
    client: web::Data<awc::Client>,
    query: web::Query<RankingRequest>,
) -> Result<maud::Markup> {
    let ranking = fetch_ranking(&client, query.date.as_ref(), query.page).await?;

    let doc = document(
        "Pixiv Proxy",
        html! {
            h1 { "Pixiv Proxy" }
            (render_options("", SearchRating::Safe, SearchOrder::DateDescending, SearchMode::TagsPartial))
            ul.search.ranking {
                @for item in ranking.contents {
                    @let url = format!("/artworks/{}", item.illust_id);
                    li {
                        div {
                            a href=(&url) {
                                @let (width, height) = util::scale_by_aspect_ratio(item.width, item.height, 200, 400);
                                @let url = &item.url;
                                img src=(&url) width=(width) height=(height) alt=(&item.title);
                            }
                        }
                        a href=(&url) { (&item.title) }
                    }
                }
            }
            @let format = format!("?date={}&p=", ranking.date);
            (render_nav(query.page, ranking.rank_total, 50, &format))
        },
        None,
    );

    Ok(doc)
}
