use actix_web::{get, web, HttpResponse, Result};
use maud::html;
use serde::Deserialize;

use crate::{
    api::{
        search::{fetch_search, SearchMode, SearchOrder, SearchRating, SearchRequest},
        user::{fetch_user_illust_ids, fetch_user_illustrations},
    },
    render::datetime::DateTimeWrapper,
};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    rss
}

#[derive(Deserialize)]
pub struct RssRequest {
    qtype: String,
    #[serde(rename = "q")]
    words: String,
    #[serde(rename = "mode", default)]
    rating: SearchRating,
    #[serde(rename = "s_mode", default)]
    mode: SearchMode,
}
pub struct RssConfig {
    pub host: String,
}

#[get("/rss")]
async fn rss(
    client: web::Data<awc::Client>,
    query: web::Query<RssRequest>,
    config: web::Data<RssConfig>,
) -> Result<HttpResponse> {
    let page = match query.qtype.as_str() {
        "author" => {
            let user_id = query.words.parse::<u64>().unwrap();
            let mut ids = fetch_user_illust_ids(&client, user_id).await?;
            ids.truncate(60);
            fetch_user_illustrations(&client, user_id, &ids).await?
        }
        _ => {
            let request = SearchRequest {
                page: 1,
                order: SearchOrder::DateDescending,
                rating: query.rating,
                mode: query.mode,
                q: None,
            };
            let search = fetch_search(&client, &query.words, &request).await?;
            search.illust_manga.data
        }
    };

    let items: Vec<::rss::Item> = page
        .iter()
        .map(|s| {
            let link = format!("{}/artworks/{}", config.host, s.id);
            let guid = ::rss::GuidBuilder::default()
                .value(link.clone())
                .permalink(true)
                .build();
            let date = chrono::DateTime::parse_from_rfc3339(&s.update_date);
            let description = match date {
                Ok(date) => {
                    let img_base = format!(
                        "{}/imageproxy/img-master/img/{}/{}",
                        config.host,
                        date.format("%Y/%m/%d/%H/%M/%S"),
                        s.id
                    );
                    html!(
                        h1 { (&s.title) }
                        p { (DateTimeWrapper(date.into())) }
                        @match s.illust_type {
                            2 => {
                                img src=(format!("{}_master1200.jpg", img_base)) alt=(s.id);
                            }
                            _ => {
                                @for i in 0..s.page_count {
                                    img src=(format!("{}_p{}_master1200.jpg", img_base, i)) alt=(i);
                                }
                            }
                        }
                    )
                }
                Err(_) => {
                    html!(
                        @let url = format!(
                            "{}{}",
                            config.host,
                            &s.url
                        );
                        img src=(url) width="250" height="250" alt=(s.id);
                    )
                }
            };
            let create_date = chrono::DateTime::parse_from_rfc3339(&s.create_date);
            let rfc2822 = match create_date {
                Ok(date) => Some(date.to_rfc2822()),
                Err(_) => None,
            };
            ::rss::ItemBuilder::default()
                .title(Some(s.title.clone()))
                .link(Some(link))
                .guid(Some(guid))
                .description(Some(description.into_string()))
                .pub_date(rfc2822)
                .build()
        })
        .collect();

    let self_url = match query.qtype.as_str() {
        "author" => {
            format!("{}/rss?type=author&q={}", config.host, query.words)
        }
        _ => {
            format!(
                "{}/rss?type=search&q={}&mode={}&s_mode={}",
                config.host, query.words, query.rating, query.mode
            )
        }
    };

    let content = ::rss::ChannelBuilder::default()
        .title(&query.words)
        .link(self_url)
        .items(items)
        .description("Pixiv RSS")
        .build();

    let content = content.to_string();

    Ok(HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(content))
}
