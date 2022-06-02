mod api;
mod imageproxy;
mod redirect;
mod render;
mod ugoira;
mod util;

use api::{
    artwork::fetch_artwork,
    comments::{fetch_comments, fetch_replies},
    common::PixivSearchResult,
    error::ApiError,
    ranking::fetch_ranking,
    search::fetch_search,
    user::{fetch_user_illust_ids, fetch_user_illustrations, fetch_user_profile},
};
use imageproxy::{handle_imageproxy, handle_stamp};
use maud::{html, PreEscaped};
use redirect::{redirect_fanbox, redirect_jump, redirect_legacy_illust};
use render::datetime::DateTimeWrapper;
use rouille::router;
use ugoira::handle_ugoira;

const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/main.css"));
const FAVICON: &[u8] = include_bytes!("../static/favicon.ico");
const SVG_PAGE_PATH: &str = "M8 3c.55 0 1 .45 1 1v5c0 .55-.45 1-1 1H3c-.55 0-1-.45-1-1h4c1.1 0 2-.9 2-2V3zM1 1h5c.55 0 1 .45 1 1v5c0 .55-.45 1-1 1H1c-.55 0-1-.45-1-1V2c0-.55.45-1 1-1z";
const SVG_PLAY_PATH: &str = "M57.5 37C35 24 35 24 35 50s0 26 22.5 13 22.5-13 0-26";
const SVG_LIKE_PATH: &str = "M2 6a2 2 0 110-4 2 2 0 010 4zm8 0a2 2 0 110-4 2 2 0 010 4zM2.11 8.89a1 1 0 011.415-1.415 3.5 3.5 0 004.95 0 1 1 0 011.414 1.414 5.5 5.5 0 01-7.778 0z";
const SVG_HEART_PATH: &str = "M16 11C15 9 13 7.5 11 7.5a5 5 0 0 0-5 5c0 5 3.25 9.25 9.75 13a.5.5 0 0 0 .5 0C22.75 21.75 26 17.5 26 12.5a5 5 0 0 0-5-5c-2 0-4 1.5-5 3.5z";
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
            (GET) (/en/tags/{tag: String}/artworks) => { handle_search(&client, request, &tag) },
            (GET) (/tags/{tag: String}/artworks) => { handle_search(&client, request, &tag) },
            (GET) (/search) => {
                let term = match request.get_param("q") {
                    Some(term) => term,
                    None => return render_error(401, "No search term"),
                };
                handle_search(&client, request, &term)
            },

            (GET) (/scroll) => { handle_scroll(&client, request) },

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

            (GET) (/comments/{id: u32}) => { handle_comments(&client, request, id) },
            (GET) (/replies/{id: u32}) => { handle_reply(&client, request, id) },
            (GET) (/stamp/{id: u32}) => { handle_stamp(&client, id) },

            (GET) (/fanbox/creator/{id: u32}) => { redirect_fanbox(&client, id) },
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

fn handle_scroll(client: &ureq::Agent, request: &rouille::Request) -> rouille::Response {
    let query = get_param_or_str!(request, "q", "");
    let order = get_param_or_str!(request, "order", "date_d");
    let mode = get_param_or_str!(request, "mode", "all");
    let page = get_param_or_num!(request, "p", 1);
    let search_mode = get_param_or_str!(request, "s_mode", "s_tag_full");

    let content = try_api!(fetch_search(
        client,
        &query,
        &order,
        &mode,
        page,
        &search_mode
    ));

    let doc = document! {
        &query,
        html! {
            h1 { (&query) }
            p { (content.illust_manga.total) }
            ul.scroll.artworks {
                @for illust in content.illust_manga.data.iter() {
                    li {
                        h2 { a href=(format!("/artworks/{}", illust.id)) { (illust.title) } }

                        @if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&illust.update_date) {
                            p { (DateTimeWrapper(date.into())) }
                            @let img_base = format!(
                                "/imageproxy/img-master/img/{}/{}",
                                date.format("%Y/%m/%d/%H/%M/%S"),
                                illust.id
                            );
                            @let (width, height) = util::scale_by_aspect_ratio(illust.width, illust.height, 900, 900);
                            @match illust.illust_type {
                                2 => {
                                    @let thumbnail = format!("{}_master1200.jpg", img_base);
                                    @let video = format!("/ugoira/{}", illust.id);
                                    video src=(&video) poster=(&thumbnail) width=(width) height=(height) controls muted loop playsinline preload="none" {}
                                }
                                _ => {
                                    img src=(format!("{}_p0_master1200.jpg", img_base)) width=(width) height=(height) alt="" loading="lazy";
                                    @if illust.page_count > 1 {
                                        details {
                                            summary {
                                                (format!("{} more...", illust.page_count - 1))
                                            }
                                            @for i in 1..illust.page_count {
                                                img src=(format!("{}_p{}_master1200.jpg", img_base, i)) alt="" loading="lazy";
                                            }
                                        }
                                    }
                                }
                            }
                        } @else {
                            img src=(util::image_to_proxy(&illust.url)) width="250" height="250" alt=(illust.id);
                        }
                    }
                }
            }
            @if content.illust_manga.total > content.illust_manga.data.len() {
                @let format = format!("scroll?q={}&mode={}&order={}&s_mode={}&p=", query, mode, order, search_mode);
                (render_nav(page, content.illust_manga.total, &format))
            }
        }
    };

    rouille::Response::html(doc.into_string())
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
            ul.search.ranking {
                @for item in ranking.contents {
                    @let url = format!("/artworks/{}", item.illust_id);
                    li {
                        div {
                            a href=(&url) {
                                @let (width, height) = util::scale_by_aspect_ratio(item.width, item.height, 200, 400);
                                @let url = util::image_to_proxy(&item.url);
                                img src=(&url) width=(width) height=(height) alt=(&item.title);
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

fn handle_search(client: &ureq::Agent, request: &rouille::Request, tag: &str) -> rouille::Response {
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
    let search = try_api!(fetch_search(client, tag, order, mode, page, search_mode));

    let docs = document! {
        tag,
        html! {
            h1 { (&tag) }
            (&search.illust_manga.total)
            @let options = format!("?q={}&mode={}&order={}&s_mode={}", tag, mode, order, search_mode);
            a href=(format!("/scroll{}&p={}", options, page)) { "Scroll..." }
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
                        (DateTimeWrapper(date.unwrap().into()))
                    }
                }
                svg viewBox="0 0 12 12" { path d=(&SVG_LIKE_PATH) fill="currentColor" {} }
                (artwork.like_count)
                svg viewBox="6 7 20 19" { path d=(&SVG_HEART_PATH) fill="currentColor" {} }
                (artwork.bookmark_count)
                svg viewBox="0 0 14 12" {
                    path d=(&SVG_EYE_OUTER_PATH) fill="currentColor" {}
                    path d=(&SVG_EYE_INNER_PATH) fill="black" {}
                }
                (artwork.view_count)
            }
            /* Images */
            div.artworks {
                @match artwork.illust_type {
                    2 => {
                        @if cfg!(feature = "ugoira") {
                            @let src = format!("/ugoira/{}", id);
                            video poster=(&image) src=(&src) controls autoplay muted loop playsinline {}
                        } @else {
                            img src=(&image) alt="";
                        }
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
            /* Comments */
            @if artwork.comment_count > 0 {
                div {
                    button endpoint=(format!("/comments/{}", id)) type="button" onclick="inject(this)" {
                        "Load Comments"
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
            /* Insert javascript if needed */
            @if artwork.comment_count > 0 {
                script {
                    (PreEscaped(include_str!("dynamic.js")))
                }
            }
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
            div {
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
                1,
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
                            host,
                            util::image_to_proxy(&s.url)
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

fn handle_comments(client: &ureq::Agent, request: &rouille::Request, id: u32) -> rouille::Response {
    let offset = get_param_or_num!(request, "offset", 0);
    let limit = get_param_or_num!(request, "limit", 100);
    let roots = match fetch_comments(client, id, offset, limit) {
        Ok(roots) => roots,
        Err(err) => {
            let doc = html! {
                "failed to read comments: "

                @match &err {
                    ApiError::External(code, message) => (*code) " " (message),
                    ApiError::Internal(message) => (500) " " (message),
                }
            };
            return rouille::Response::html(doc.into_string());
        }
    };
    let doc = html! {
        ul.comments {
            @for comment in roots.comments {
                (&comment)
            }
        }
        @if roots.has_next {
            button endpoint=(format!("/comments/{}?offset={}&limit={}", id, offset + limit, limit)) onclick="inject(this)" {
                "Load more..."
            }
        }
    };
    rouille::Response::html(doc.into_string())
}

fn handle_reply(client: &ureq::Agent, request: &rouille::Request, id: u32) -> rouille::Response {
    let page = get_param_or_num!(request, "page", 1);
    let replies = match fetch_replies(client, id, page) {
        Ok(replies) => replies,
        Err(err) => {
            let doc = html! {
                "failed to read comments: "

                @match &err {
                    ApiError::External(code, message) => (*code) " " (message),
                    ApiError::Internal(message) => (500) " " (message),
                }
            };
            return rouille::Response::html(doc.into_string());
        }
    };
    let doc = html! {
        @if replies.has_next {
            button endpoint=(format!("/replies/{}?page={}", id, page + 1)) onclick="inject(this, false)" {
                "Load older replies"
            }
        }
        ul.comments {
            @for comment in replies.comments.iter().rev() {
                (comment)
            }
        }
    };
    rouille::Response::html(doc.into_string())
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
                                svg { use href="#page" {} }
                                (artwork.page_count)
                            }
                        }
                        @if artwork.illust_type == 2 {
                            svg.search__play { use href="#play" {} }
                        }
                        img src=(&img) width="200" height="200" alt=(&artwork.title);
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
            input type="text" name="q" placeholder="Keywords..." value=(&tag) required;
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
