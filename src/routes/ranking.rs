use maud::html;

use crate::{
    api::{
        error::ApiError,
        ranking::fetch_ranking,
        search::{SearchMode, SearchOrder, SearchRating},
    },
    get_param_or_num,
    render::{document::document, nav::render_nav, search::render_options},
    util,
};

pub fn ranking(
    client: &ureq::Agent,
    query: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let date = query.get_param("date");
    let page = get_param_or_num!(query, "p", 1);
    let ranking = fetch_ranking(client, date.as_ref(), page)?;

    let document = document(
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
                                img src=(&url) width=(width) height=(height) alt="";
                            }
                        }
                        a href=(&url) { (&item.title) }
                    }
                }
            }
            @let format = format!("?date={}&p=", ranking.date);
            (render_nav(page, ranking.rank_total, 50, &format))
        },
        None,
    );

    Ok(rouille::Response::html(document.into_string()))
}
