mod api;
mod imageproxy;
mod redirect;
mod ugoira;
mod util;

use api::{
    artwork::fetch_artwork,
    common::PixivSearchResult,
    error::ApiError,
    ranking::fetch_ranking,
    search::fetch_search,
    user::{fetch_user_illust_ids, fetch_user_illustrations, fetch_user_profile},
};
use imageproxy::handle_imageproxy;
use maud::{html, PreEscaped};
use redirect::{redirect_jump, redirect_legacy_illust};
use rouille::router;
use ugoira::handle_ugoira;

const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/main.css"));
const FAVICON: &[u8] = include_bytes!("../static/favicon.ico");
const SVG_PAGE_PATH: &str = "M8,3C8.55,3 9,3.45 9,4L9,9C9,9.55 8.55,10 8,10L3,10C2.45,10 2,9.55 2,9L6,9C7.1,9 8,8.1 8,7L8,3Z M1,1L6,1C6.55,1 7,1.45 7,2L7,7C7,7.55 6.55,8 6,8L1,8C0.45,8 0,7.55 0,7L0,2C0,1.45 0.45,1 1,1Z";
const SVG_PLAY_PATH: &str = "M 57.5 37 C 35 24, 35 24, 35 50 S 35 76, 57.5 63 S 80 50, 57.5 37";
const SVG_LIKE_PATH: &str = "M2 6a2 2 0 110-4 2 2 0 010 4zm8 0a2 2 0 110-4 2 2 0 010 4zM2.11 8.89a1 1 0 011.415-1.415 3.5 3.5 0 004.95 0 1 1 0 011.414 1.414 5.5 5.5 0 01-7.778 0z";
const SVG_HEART_PATH: &str = "M9,0.75 C10.5,0.75 12,2 12,3.75 C12,6.5 10,9.25 6.25,11.5L6.25,11.5 C6,11.5 6,11.5 5.75,11.5C2,9.25 0,6.75 0,3.75 C1.1324993e-16,2 1.5,0.75 3,0.75C4,0.75 5.25,1.5 6,2.75 C6.75,1.5 9,0.75 9,0.75 Z";
const SVG_EYE_OUTER_PATH: &str =
    "M0 6c2-3.333 4.333-5 7-5s5 1.667 7 5c-2 3.333-4.333 5-7 5S2 9.333 0 6z";
const SVG_EYE_INNER_PATH: &str =
    "M7 8.5a2.5 2.5 0 110-5 2.5 2.5 0 010 5zm0-1a1.5 1.5 0 100-3 1.5 1.5 0 000 3z";

fn document(title: &str, content: maud::Markup, head: Option<maud::Markup>) -> maud::Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { (title) }
                style { (CSS) }
                meta name="viewport" content="width=device-width, initial-scale=1";
                @if head.is_some() { (head.unwrap()) }
            }
            body {
                main { (content) }
                footer { div { a href="/" { "Home" } " - " a href="/about" { "About" } } }
            }
        }
    }
}
macro_rules! document {
    ($title:expr, $content:expr, $head:expr) => {
        document($title, $content, Some($head))
    };
    ($title:expr, $content:expr) => {
        document($title, $content, None)
    };
}

fn main() {
    let mut pargs = pico_args::Arguments::from_env();
    let port: u32 = pargs.value_from_str("--port").unwrap();
    let host: String = pargs.value_from_str("--host").unwrap();
    let cookie: String = pargs.value_from_str("--cookie").unwrap();

    /* Construct http client */
    let client: ureq::Agent = {
        /* Load tls certificate */
        let certs = rustls_native_certs::load_native_certs().expect("Could not load certs!");

        let mut root_store = rustls::RootCertStore::empty();
        for cert in certs {
            root_store
                .add(&rustls::Certificate(cert.0))
                .expect("Could not add cert!");
        }
        let tls_config = std::sync::Arc::new(
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );

        /* Add default headers */
        struct PixivDefaultHeaders {
            referer: String,
            cookie: String,
        }

        impl ureq::Middleware for PixivDefaultHeaders {
            fn handle(
                &self,
                request: ureq::Request,
                next: ureq::MiddlewareNext,
            ) -> Result<ureq::Response, ureq::Error> {
                let request = request
                    .set("Referer", &self.referer)
                    .set("Cookie", &self.cookie);
                next.handle(request)
            }
        }

        let middleware = PixivDefaultHeaders {
            referer: "https://pixiv.net/".to_string(),
            cookie,
        };

        /* Build client */
        ureq::AgentBuilder::new()
            .tls_config(tls_config)
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:99.0) Gecko/20100101 Firefox/99.0",
            )
            .middleware(middleware)
            .redirects(0)
            .build()
    };

    let address = format!("0.0.0.0:{}", port);
    rouille::start_server(&address, move |request| {
        router!(request,
            (GET) (/) => { handle_ranking(&client, request) },
            (GET) (/en/) => { handle_ranking(&client, request) },
            (GET) (/en/tags/{tag: String}/artworks) => { handle_tags(&client, request, &tag) },
            (GET) (/tags/{tag: String}/artworks) => { handle_tags(&client, request, &tag) },
            (GET) (/search) => {
                let term = match request.get_param("q") {
                    Some(term) => term,
                    None => return render_error(401, "No search term"),
                };
                handle_tags(&client, request, &term)
            },

            /* user */
            (GET) (/en/users/{id: String}/artworks) => { handle_user(&client, request, &id) },
            (GET) (/users/{id: String}/artworks) => { handle_user(&client, request, &id) },
            (GET) (/en/users/{id: String}) => { handle_user(&client, request, &id) },
            (GET) (/users/{id: String}) => { handle_user(&client, request, &id) },

            /* artwork */
            (GET) (/en/artworks/{id: u32}) => { handle_artwork(&client, id) },
            (GET) (/artworks/{id: u32}) => { handle_artwork(&client, id) },

            (GET) (/ugoira/{id: u32}) => { handle_ugoira(&client, id) },
            (GET) (/rss) => { handle_rss(&client, request, &host) },
            (GET) (/about) => { render_about() },
            _ => {
                /* Just matching manually now... */
                let url = request.url();
                if url.starts_with("/imageproxy/") {
                    return handle_imageproxy(&client, &url);
                }
                match url.as_str() {
                    "/favicon.ico" => {
                        rouille::Response::from_data("image/x-icon", FAVICON)
                            .with_public_cache(365 * 24 * 60 * 60)
                    },
                    "/jump.php" => { redirect_jump(request) },
                    "/member_illust.php" => { redirect_legacy_illust(request) },
                    _ => { render_error(404, "Endpoint not found!") }
                }
            }
        )
    });
}

macro_rules! try_api {
    ($query:expr) => {
        match ($query) {
            Ok(res) => res,
            Err(err) => {
                return render_api_error(&err);
            }
        }
    };
}

macro_rules! get_param_or_str {
    ($request:expr, $name:expr, $default:expr) => {{
        let option = $request.get_param($name);
        match option {
            Some(value) => value,
            None => $default.to_string(),
        }
    }};
}
macro_rules! get_param_or_num {
    ($request:expr, $name:expr, $default:expr) => {{
        let option = $request.get_param($name);
        match option {
            Some(value) => value.parse::<u32>().unwrap_or($default),
            None => $default,
        }
    }};
}

fn handle_ranking(client: &ureq::Agent, request: &rouille::Request) -> rouille::Response {
    let date = request.get_param("date");
    let page = get_param_or_num!(request, "p", 1);

    let ranking = try_api!(fetch_ranking(client, date, page));

    let doc = document! {
        "Pixiv Proxy",
        html! {
            h1 { "Pixiv Proxy" }
            (render_options("", "safe", "date_d", "s_tag"))
            ul.search {
                @for item in ranking.contents {
                    @let url = format!("/artworks/{}", item.illust_id);
                    li style="height:auto;text-align:center;" {
                        div {
                            a href=(&url) {
                                @let ratio = item.height as f32 / item.width as f32;
                                @let (width, height) = if ratio < 2.0 {
                                    (200, f32::ceil(ratio * 200.0) as u32)
                                } else {
                                    (f32::ceil(400.0 / ratio) as u32, 400)
                                };
                                @let url = util::image_to_proxy(&item.url);
                                img src=(&url) width=(width) height=(height);
                            }
                        }
                        a href=(&url) { (&item.title) }
                    }
                }
            }
            @let format = format!("?date={}&p=", ranking.date);
            (render_nav(page, ranking.rank_total, &format))
        }
    };

    rouille::Response::html(doc.into_string())
}

fn handle_tags(client: &ureq::Agent, request: &rouille::Request, tag: &str) -> rouille::Response {
    let order = get_param_or_str!(request, "order", "date_d");
    let mode = get_param_or_str!(request, "mode", "all");
    let page = get_param_or_num!(request, "p", 1);
    let search_mode = get_param_or_str!(request, "s_mode", "s_tag_full");

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
    let search = try_api!(fetch_search(
        client,
        tag,
        order,
        mode,
        &page.to_string(),
        search_mode
    ));

    let docs = document! {
        tag,
        html! {
            h1 { (&tag) }
            (&search.illust_manga.total)
            (render_options(tag, mode, order, search_mode))
            (render_list(&search.illust_manga.data))
            @if search.illust_manga.total > 60 {
                @let format = format!("/tags/{}/artworks?order={}&mode={}&s_mode={}&p=", tag, order, mode, search_mode);
                (render_nav(page, search.illust_manga.total, &format))
            }
        }
    };

    rouille::Response::html(docs.into_string())
}

fn handle_artwork(client: &ureq::Agent, id: u32) -> rouille::Response {
    let artwork = try_api!(fetch_artwork(client, &id.to_string()));

    let image = util::image_to_proxy(&artwork.urls.original);
    let date = chrono::DateTime::parse_from_rfc3339(&artwork.create_date);

    let docs = document! {
        &artwork.illust_title,
        html! {
            /* Title */
            h1 { (&artwork.illust_title) }
            /* Author */
            @let link = format!("/users/{}", percent_encoding::utf8_percent_encode(&artwork.user_id, percent_encoding::NON_ALPHANUMERIC));
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
                        (date.unwrap().format("%Y-%m-%d %H:%M:%S -").to_string())
                    }
                }
                svg viewBox="0 0 12 12" { path d=(&SVG_LIKE_PATH) fill="currentColor" {} }
                (artwork.like_count)
                svg viewBox="0 0 12 12" { path d=(&SVG_HEART_PATH) fill="currentColor" {} }
                (artwork.bookmark_count)
                svg viewBox="0 0 14 12" {
                    path d=(&SVG_EYE_OUTER_PATH) fill="currentColor" {}
                    path d=(&SVG_EYE_INNER_PATH) fill="black" {}
                }
                (artwork.view_count)
            }
            /* Images */
            div.illust__images {
                @match artwork.illust_type {
                    2 => {
                        @let src = format!("/ugoira/{}", id);
                        video poster=(&image) src=(&src) controls="" autoplay="" loop="" muted="" {}
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
        },
        html! {
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
                    meta name="twitter:card" content="summary_large_image";
                }
            }
            @let description = util::truncate(&artwork.description, 200);
            meta property="og:description" content=(&description);
        }
    };

    rouille::Response::html(docs.into_string())
}

fn handle_user(
    client: &ureq::Agent,
    request: &rouille::Request,
    user_id: &str,
) -> rouille::Response {
    let user = try_api!(fetch_user_profile(client, user_id));
    let ids = try_api!(fetch_user_illust_ids(client, user_id));

    let page = get_param_or_num!(request, "p", 1);
    let count = ids.len();
    let start = (page - 1) * 60;
    let end = std::cmp::min(start + 60, count as u32);
    let slice = &ids[start as usize..end as usize];

    let elements = try_api!(fetch_user_illustrations(client, user_id, slice));

    let image = util::image_to_proxy(&user.image_big);

    let doc = document! {
        &user.name,
        html! {
            header.author {
                img.logo src=(&image) alt=(&user.name) width="170";
                h1 { (&user.name) }
                @if !user.comment_html.is_empty() {
                    p { (PreEscaped(&user.comment_html)) }
                }
            }
            section {
                (render_list(&elements))
            }
            @if count > 60 {
                @let format = format!("/users/{}?p=", user_id);
                (render_nav(page, count, &format))
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

fn handle_rss(client: &ureq::Agent, request: &rouille::Request, host: &str) -> rouille::Response {
    let query_type = get_param_or_str!(request, "type", "search");
    let query_words = get_param_or_str!(request, "q", "");
    let query_mode = get_param_or_str!(request, "mode", "all");
    let query_search_mode = get_param_or_str!(request, "s_mode", "s_tag_full");

    let page = match query_type.as_str() {
        "author" => {
            let ids = try_api!(fetch_user_illust_ids(client, &query_words));
            try_api!(fetch_user_illustrations(client, &query_words, &ids))
        }
        _ => {
            let search = try_api!(fetch_search(
                client,
                &query_words,
                "date_d",
                &query_mode,
                "1",
                &query_search_mode,
            ));
            search.illust_manga.data
        }
    };

    let items: Vec<rss::Item> = page
        .iter()
        .map(|s| {
            let link = format!("{}/artworks/{}", host, s.id);
            let guid = rss::GuidBuilder::default()
                .value(link.clone())
                .permalink(true)
                .build();
            let date = chrono::DateTime::parse_from_rfc3339(&s.update_date);
            let description = match date {
                Ok(date) => {
                    let img_base = format!(
                        "{}/imageproxy/img-master/img/{}/{}",
                        host,
                        date.format("%Y/%m/%d/%H/%M/%S"),
                        s.id
                    );
                    html!(
                        h1 { (&s.title) }
                        p { (date.format("%Y-%m-%d %H:%M:%S").to_string()) }
                        @match s.illust_type {
                            2 => {
                                img src=(format!("{}_master1200.jpg", img_base));
                            }
                            _ => {
                                @for i in 0..s.page_count {
                                    img src=(&format!("{}_p{}_master1200.jpg", img_base, i));
                                }
                            }
                        }
                    )
                }
                Err(_) => {
                    html!(
                        @let url = format!(
                            "{}{}",
                            host,
                            util::image_to_proxy(&s.url)
                        );
                        img src=(url) width="250" height="250";
                    )
                }
            };
            let create_date = chrono::DateTime::parse_from_rfc3339(&s.create_date);
            let rfc2822 = match create_date {
                Ok(date) => Some(date.to_rfc2822()),
                Err(_) => None,
            };
            rss::ItemBuilder::default()
                .title(Some(s.title.clone()))
                .link(Some(link))
                .guid(Some(guid))
                .description(Some(description.into_string()))
                .pub_date(rfc2822)
                .build()
        })
        .collect();

    let self_url = match query_type.as_str() {
        "author" => {
            format!("{}/rss?type=author&q={}", host, query_words)
        }
        _ => {
            format!(
                "{}/rss?type=search&q={}&mode={}&s_mode={}",
                host, query_words, query_mode, query_search_mode
            )
        }
    };

    let content = rss::ChannelBuilder::default()
        .title(query_words)
        .link(self_url)
        .items(items)
        .description("Pixiv RSS")
        .build();

    let content = content.to_string();

    rouille::Response {
        status_code: 200,
        headers: vec![(
            "Content-Type".into(),
            "application/rss+xml; charset=utf-8".into(),
        )],
        data: rouille::ResponseBody::from_string(content),
        upgrade: None,
    }
}


fn render_list(list: &[PixivSearchResult]) -> maud::Markup {
    html! {
        svg style="display:none" {
            defs {
                symbol id="page" viewBox="0 0 10 10" { path d=(SVG_PAGE_PATH) {} }
                symbol id="play" viewBox="0 0 100 100" {
                    circle fill="#1f1f1fd0" cx="50" cy="50" r="50" {}
                    path fill="#fff" d=(SVG_PLAY_PATH) {}
                }
            }
        }
        ul.search {
            @for artwork in list {
                @let link = format!("/artworks/{}", artwork.id);
                @let img = util::image_to_proxy(&artwork.url);
                li {
                    a href=(&link) {
                        @if artwork.r18 == 1 {
                            div.search__hover.search__warn { "R-18" }
                        }
                        @if artwork.page_count > 1 {
                            div.search__hover.search__count {
                                svg { use href="#page"; }
                                (artwork.page_count)
                            }
                        }
                        @if artwork.illust_type == 2 {
                            svg.search__play { use href="#play"; }
                        }
                        img src=(&img) width="200" height="200";
                    }
                    a href=(&link) { (&artwork.title) }
                }
            }
        }
    }
}

fn render_options(tag: &str, mode: &str, order: &str, search_mode: &str) -> maud::Markup {
    fn make_option(name: &str, value: &str, mode: &str) -> maud::Markup {
        html! {
            @if mode == value {
                option value=(&value) selected { (&name) }
            } @else {
                option value=(&value) { (&name) }
            }
        }
    }

    html! {
        form action="/search" method="get" {
            input type="text" name="q" placeholder="Keywords..." value=(&tag);
            select name="mode" {
                (make_option("All", "all", mode));
                (make_option("Safe", "safe", mode));
                (make_option("R-18", "r18", mode));
            }
            select name="order" {
                (make_option("By Upload Date (Newest)", "date_d", order));
                (make_option("By Upload Date (Oldest)", "date", order));
                (make_option("By Popularity (All)", "popular_d", order));
                (make_option("By Popularity (Male)", "popular_male_d", order));
                (make_option("By Popularity (Female)", "popular_female_d", order));
            }
            select name="s_mode" {
                (make_option("Tags (partial match)", "s_tag", search_mode))
                (make_option("Tags (perfect match)", "s_tag_full", search_mode))
                (make_option("Title, Caption", "s_tc", search_mode))
            }
            button type="submit" { "Search" }
        }
    }
}

fn render_nav(current_page: u32, count: usize, template: &str) -> maud::Markup {
    html! {
        @if count > 60 {
            nav {
                @let min = 1;
                @let max = count / 60 + 1;
                @let nav_start = std::cmp::max(min as i32, current_page as i32 - 3);
                @let nav_end = std::cmp::min(max as i32, nav_start + 7);
                @for page in nav_start..nav_end {
                    @if page as u32 == current_page {
                        span { (page) }
                    } @else {
                        @let link = format!("{}{}", template, page);
                        a href=(&link) { (page) }
                    }
                }
            }
        }
    }
}

fn render_error(code: u16, message: &str) -> rouille::Response {
    let title = format!("{}", code);

    let doc = document! {
        &title,
        html! {
            h1 { (&title) }
            p { (message) }
            p { a href="/" { "Go home" } }
        }
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
        }
    };

    rouille::Response::html(doc.into_string())
}
