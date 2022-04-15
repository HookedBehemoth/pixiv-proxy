mod api;
mod util;

use api::{
    artwork::fetch_artwork,
    common::PixivSearchResult,
    error::ApiError,
    search::fetch_search,
    user::{fetch_user_illust_ids, fetch_user_illustrations, fetch_user_profile},
};
use maud::{html, PreEscaped};
use rouille::router;

use rustls::Certificate;
use std::borrow::Cow;
use std::sync::Arc;
use ureq::{Agent, AgentBuilder};

const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/main.css"));
const FAVICON: &[u8] = include_bytes!("../static/favicon.ico");
const SVG_PAGE_PATH: &str = "M8,3 C8.55,3 9,3.45 9,4 L9,9 C9,9.55 8.55,10 8,10 L3,10 C2.45,10 2,9.55 2,9 L6,9 C7.1,9 8,8.1 8,7 L8,3 Z M1,1 L6,1 C6.55,1 7,1.45 7,2 L7,7 C7,7.55 6.55,8 6,8 L1,8 C0.45,8 0,7.55 0,7 L0,2 C0,1.45 0.45,1 1,1 Z";

macro_rules! document {
    ($title:expr, $content:expr, $( $head:expr )? ) => {
        html! {
            html lang="en" {
                head {
                    title { ($title) }
                    style { (CSS) }
                    meta name="viewport" content="width=device-width, initial-scale=1";
                    $( ($head) )?
                }
                body {
                    main { ($content) }
                    footer { div { a href="/" { "Home" } " - " a href="/about" { "About" } } }
                }
            }
        }
    };
}

fn main() {
    let client: Agent = {
        let certs = rustls_native_certs::load_native_certs().expect("Could not load certs!");

        let mut root_store = rustls::RootCertStore::empty();
        for cert in certs {
            root_store
                .add(&Certificate(cert.0))
                .expect("Could not add cert!");
        }
        let tls_config = Arc::new(
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );

        AgentBuilder::new()
            .tls_config(tls_config)
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:97.0) Gecko/20100101 Firefox/97.0",
            )
            .redirects(0)
            .build()
    };

    rouille::start_server("0.0.0.0:8080", move |request| {
        router!(request,
            (GET) (/) => {
                render_search(&client, "けものフレンズ", "popular_d", "safe", 1, "s_tag_full")
            },
            (GET) (/en/) => {
                render_search(&client, "けものフレンズ", "popular_d", "safe", 1, "s_tag_full")
            },
            (GET) (/en/tags/{tag: String}/artworks) => {
                handle_tags(&client, &request, &tag)
            },
            (GET) (/tags/{tag: String}/artworks) => {
                handle_tags(&client, &request, &tag)
            },
            (GET) (/en/users/{id: String}/artworks) => {
                handle_user(&client, &request, &id)
            },
            (GET) (/users/{id: String}/artworks) => {
                handle_user(&client, &request, &id)
            },
            (GET) (/en/users/{id: String}) => {
                handle_user(&client, &request, &id)
            },
            (GET) (/users/{id: String}) => {
                handle_user(&client, &request, &id)
            },
            (GET) (/en/artworks/{id: u32}) => {
                handle_artwork(&client, id)
            },
            (GET) (/artworks/{id: u32}) => {
                handle_artwork(&client, id)
            },
            (GET) (/about) => {
                render_about()
            },
            _ => {
                /* Just matching manually now... */
                let url = request.url();
                if url.starts_with("/imageproxy/") {
                    handle_imageproxy(&client, &url)
                        .with_public_cache(365 * 24 * 60 * 60)
                } else if url == "/favicon.ico" {
                    rouille::Response::from_data("image/x-icon", FAVICON)
                        .with_public_cache(365 * 24 * 60 * 60)
                } else if url == "/jump.php" {
                    let destination = request.raw_query_string().as_bytes();
                    let destination = percent_encoding::percent_decode(destination)
                        .decode_utf8_lossy()
                        .into_owned();
                    rouille::Response {
                        status_code: 301,
                        headers: vec![("Location".into(), destination.into())],
                        data: rouille::ResponseBody::empty(),
                        upgrade: None,
                    }

                } else {
                    rouille::Response::empty_404()
                }
            }
        )
    });
}

fn handle_imageproxy(client: &ureq::Agent, path: &str) -> rouille::Response {
    let url = format!("https://i.pximg.net/{}", &path[12..]);
    let response = client
        .get(&url)
        .set("Referer", "https://pixiv.net/")
        .set(
            "Cookie",
            &std::env::args().nth(1).expect("PIXIV_COOKIE must be set"),
        )
        .call()
        .unwrap();

    let headers = response
        .headers_names()
        .iter()
        .filter(|&s| s != "cookies")
        .map(|s| {
            (
                Cow::from(s.clone()),
                Cow::from(response.header(s).unwrap_or("").to_string()),
            )
        })
        .collect();

    let reader = match response
        .header("Content-Length")
        .map(|s| s.parse::<usize>())
    {
        Some(Ok(len)) => rouille::ResponseBody::from_reader_and_size(response.into_reader(), len),
        _ => rouille::ResponseBody::from_reader(response.into_reader()),
    };

    rouille::Response {
        status_code: 200,
        headers: headers,
        data: reader,
        upgrade: None,
    }
}

fn handle_tags(client: &ureq::Agent, request: &rouille::Request, tag: &str) -> rouille::Response {
    let order = request.get_param("order").unwrap_or("date_d".to_string());
    let mode = request.get_param("mode").unwrap_or("all".to_string());
    let page = request
        .get_param("p")
        .map(|p| p.parse::<u32>().unwrap_or(1))
        .unwrap_or(1);
    let search_mode = request
        .get_param("s_mode")
        .unwrap_or("s_tag_full".to_string());

    render_search(client, tag, &order, &mode, page, &search_mode)
}

fn render_search(
    client: &ureq::Agent,
    tag: &str,
    order: &str,
    mode: &str,
    page: u32,
    search_mode: &str,
) -> rouille::Response {
    let search = match fetch_search(client, tag, order, mode, &page.to_string(), search_mode) {
        Ok(search) => search,
        Err(err) => {
            return render_api_error(&err);
        }
    };

    let docs = document! {
        &tag,
        html! {
            h1 { (&tag) }
            p1 { (&search.illust_manga.total) }
            (render_list(&search.illust_manga.data, page, search.illust_manga.total, "tags", &tag))
        },
    };

    rouille::Response::html(docs.into_string())
}

fn handle_artwork(client: &ureq::Agent, id: u32) -> rouille::Response {
    let artwork = match fetch_artwork(client, &id.to_string()) {
        Ok(artwork) => artwork,
        Err(err) => {
            return render_api_error(&err);
        }
    };

    let image = util::image_to_proxy(&artwork.urls.original);

    let docs = document! {
        &artwork.illust_title,
        html! {
            /* Title */
            h1 { (&artwork.illust_title) }
            /* Author */
            @let link = format!("/users/{}", &artwork.user_id);
            p class="illust__author" { a href=(&link) { (&artwork.user_name) } }
            /* Description */
            @if !artwork.description.is_empty() {
                p { noscript { (PreEscaped(&artwork.description)) } }
            }
            /* Tags */
            (artwork.tags)
            /* Meta */
            p class="illust__meta" { (format!("likes: {}, favorites: {}, views: {}", artwork.like_count, artwork.bookmark_count, artwork.view_count)) }
            /* Images */
            ul class="illust__images" {
                @for url in std::iter::once(image.clone())
                    .chain(
                        (1..artwork.page_count).map(|i|
                            image.clone().replace("_p0.", &format!("_p{}.", i))
                        )
                    ) {
                    li { img src=(&url) alt=(&artwork.alt); }
                }
            }
        },
        html! {
            meta property="og:title" content=(&artwork.illust_title);
            meta property="og:type" content="image";
            @let description = util::truncate(&artwork.description, 100);
            meta property="og:description" content=(&description);
            meta property="og:url" content=(&format!("/artworks/{}", id));
            meta property="og:image" content=(&image);
            meta property="og:image:width" content=(&artwork.width);
            meta property="og:image:height" content=(&artwork.height);
        }
    };

    rouille::Response::html(docs.into_string())
}

fn handle_user(
    client: &ureq::Agent,
    request: &rouille::Request,
    user_id: &str,
) -> rouille::Response {
    let user = match fetch_user_profile(client, user_id) {
        Ok(user) => user,
        Err(err) => {
            return render_api_error(&err);
        }
    };

    let page = request
        .get_param("p")
        .map(|p| p.parse::<u32>().unwrap_or(1))
        .unwrap_or(1);

    let ids = match fetch_user_illust_ids(client, user_id) {
        Ok(ids) => ids,
        Err(err) => {
            return render_api_error(&err);
        }
    };

    let count = ids.len();
    let start = (page - 1) * 60;
    let end = std::cmp::min(start + 60, count as u32);
    let slice = &ids[start as usize..end as usize];

    let elements = match fetch_user_illustrations(client, user_id, slice) {
        Ok(elements) => elements,
        Err(err) => {
            return render_api_error(&err);
        }
    };

    let mut elements: Vec<(u32, PixivSearchResult)> = elements.works.into_iter().collect();
    elements.sort_unstable_by_key(|s| s.0);
    elements.reverse();
    let elements: Vec<PixivSearchResult> = elements.into_iter().map(|(_, s)| s).collect();

    let image = util::image_to_proxy(&user.image_big);

    let doc = document! {
        &user.name,
        html! {
            header class="author" {
                img class="logo" src=(&image) alt=(&user.name) width="170";
                h1 { (&user.name) }
                p { noscript { (PreEscaped(&user.comment_html)) } }
            }
            section {
                (render_list(&elements, page, count, "users", user_id))
            }
        },
        html! {
            meta property="og:title" content=(&user.name);
            meta property="og:type" content="image";
            @let description = format!("{} Images", count);
            meta property="og:description" content=(&description);
            meta property="og:url" content=(&format!("/users/{}", user_id));
            meta property="og:image" content=(&image);
            meta property="og:image:width" content="170";
            meta property="og:image:height" content="170";
        }
    };

    rouille::Response::html(doc.into_string())
}

fn render_list(
    list: &[PixivSearchResult],
    page: u32,
    count: usize,
    nav_type: &str,
    nav_index: &str,
) -> maud::Markup {
    html! {
        ul class="search" {
            @for artwork in list {
                @let link = format!("/en/artworks/{}", artwork.id);
                @let img = util::image_to_proxy(&artwork.url);
                li class="search__item" {
                    a href=(&link) {
                        @if artwork.r18 == 1 {
                            div class="search__hover search__warn" { "R-18" }
                        }
                        @if artwork.page_count > 1 {
                            div class="search__hover search__count" {
                                svg class="search__count__icon" viewBox="0 0 10 10" {
                                    path d=(SVG_PAGE_PATH);
                                }
                                (artwork.page_count)
                            }
                        }
                        img class="search__image" src=(&img) width="250" height="250";
                    }
                    a href=(&link) class="search__text" { (&artwork.title) }
                }
            }
        }
        @if count > 60 {
            nav {
                @let min = 1;
                @let max = count / 60 + 1;
                @let nav_start = std::cmp::max(min as i32, page as i32 - 3);
                @let nav_end = std::cmp::min(max as i32, nav_start + 7);
                @for page in nav_start..nav_end {
                    @let link = format!("/{}/{}/artworks?p={}", nav_type, nav_index, page);
                    a href=(&link) { (&page) }
                }
            }
        }
    }
}

fn render_error(code: u16, message: &str) -> rouille::Response {
    let title = format!("{} - {}", code, message);

    let doc = document! {
        &title,
        html! {
            h1 { (&title) }
            p { a href="/" { "Go home" } }
        },
    };

    rouille::Response::html(doc.into_string()).with_status_code(code)
}

fn render_api_error(err: &ApiError) -> rouille::Response {
    match &err {
        ApiError::External(code, message) => render_error(*code, message),
        ApiError::Internal(message) => render_error(500, message),
    }
}

fn render_about() -> rouille::Response {
    let doc = document! {
        "About",
        html! {
            h1 { "About" }
            p { "This is a simple Pixiv API client written in Rust." }
            p { "The source code is available on " a href="https://github.com/HookedBehemoth/pixiv-proxy" { "GitHub" } "." }
        },
    };

    rouille::Response::html(doc.into_string())
}
