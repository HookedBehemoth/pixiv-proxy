use maud::html;
use std::str::FromStr;

use crate::{
    api::{
        error::ApiError,
        search::{fetch_search, SearchMode, SearchOrder, SearchRating, SearchRequest},
        user::{fetch_user_illust_ids, fetch_user_illustrations},
    },
    get_param_or_enum,
    render::datetime::DateTimeWrapper,
};

pub struct RssConfig {
    pub host: String,
}

pub fn rss(
    client: &ureq::Agent,
    query: &rouille::Request,
    config: &RssConfig,
) -> Result<rouille::Response, ApiError> {
    let words = query
        .get_param("q")
        .ok_or_else(|| ApiError::External(403, "Missing Parameter".into()))?;
    let qtype = query.get_param("qtype").unwrap();
    let rating = get_param_or_enum!(query, "rating", SearchRating, SearchRating::All);
    let mode = get_param_or_enum!(query, "mode", SearchMode, SearchMode::TagsPerfect);
    let page = match qtype.as_str() {
        "author" => {
            let user_id = words.parse::<u64>().unwrap();
            let mut ids = fetch_user_illust_ids(client, user_id)?;
            ids.truncate(60);
            fetch_user_illustrations(client, user_id, &ids)?
        }
        _ => {
            let request = SearchRequest {
                page: 1,
                order: SearchOrder::DateDescending,
                rating,
                mode,
            };
            let search = fetch_search(client, &words, &request)?;
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

    let self_url = match qtype.as_str() {
        "author" => {
            format!("{}/rss?type=author&q={}", config.host, words)
        }
        _ => {
            format!(
                "{}/rss?type=search&q={}&mode={}&s_mode={}",
                config.host, words, rating, mode
            )
        }
    };

    let content = ::rss::ChannelBuilder::default()
        .title(&words)
        .link(self_url)
        .items(items)
        .description("Pixiv RSS")
        .build();

    let content = content.to_string();

    Ok(rouille::Response {
        status_code: 200,
        headers: vec![(
            "Content-Type".into(),
            "application/rss+xml; charset=utf-8".into(),
        )],
        data: rouille::ResponseBody::from_string(content),
        upgrade: None,
    })
}
