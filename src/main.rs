mod api;
mod util;

use api::{
    artwork::fetch_artwork,
    common::PixivSearchResult,
    error::ApiError,
    search::fetch_search,
    ugoira::{fetch_ugoira_meta, UgoiraFrame},
    user::{fetch_user_illust_ids, fetch_user_illustrations, fetch_user_profile},
};
use maud::{html, PreEscaped};
use rouille::router;

use rustls::Certificate;
use std::{
    borrow::Cow,
    io::{Cursor, Read, Seek, Write},
};
use std::{io::BufReader, sync::Arc};
use ureq::{Agent, AgentBuilder};

const BASE_URL: &str = "https://illegalesachen.de";
const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/main.css"));
const FAVICON: &[u8] = include_bytes!("../static/favicon.ico");
const SVG_PAGE_PATH: &str = "M8,3C8.55,3 9,3.45 9,4L9,9C9,9.55 8.55,10 8,10L3,10C2.45,10 2,9.55 2,9L6,9C7.1,9 8,8.1 8,7L8,3Z M1,1L6,1C6.55,1 7,1.45 7,2L7,7C7,7.55 6.55,8 6,8L1,8C0.45,8 0,7.55 0,7L0,2C0,1.45 0.45,1 1,1Z";
const SVG_LIKE_PATH: &str = "M2 6a2 2 0 110-4 2 2 0 010 4zm8 0a2 2 0 110-4 2 2 0 010 4zM2.11 8.89a1 1 0 011.415-1.415 3.5 3.5 0 004.95 0 1 1 0 011.414 1.414 5.5 5.5 0 01-7.778 0z";
const SVG_HEART_PATH: &str = "M9,0.75 C10.5,0.75 12,2 12,3.75 C12,6.5 10,9.25 6.25,11.5L6.25,11.5 C6,11.5 6,11.5 5.75,11.5C2,9.25 0,6.75 0,3.75 C1.1324993e-16,2 1.5,0.75 3,0.75C4,0.75 5.25,1.5 6,2.75 C6.75,1.5 9,0.75 9,0.75 Z";
const SVG_EYE_OUTER_PATH: &str =
    "M0 6c2-3.333 4.333-5 7-5s5 1.667 7 5c-2 3.333-4.333 5-7 5S2 9.333 0 6z";
const SVG_EYE_INNER_PATH: &str =
    "M7 8.5a2.5 2.5 0 110-5 2.5 2.5 0 010 5zm0-1a1.5 1.5 0 100-3 1.5 1.5 0 000 3z";

macro_rules! document {
    ($title:expr, $content:expr, $( $head:expr )? ) => {
        html! {
            (maud::DOCTYPE)
            html lang="en" {
                head {
                    meta charset="utf-8";
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
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:99.0) Gecko/20100101 Firefox/99.0",
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
            (GET) (/search) => {
                let term = match request.get_param("q") {
                    Some(term) => term,
                    None => return render_error(401, "No search term"),
                };
                handle_tags(&client, &request, &term)
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
            (GET) (/ugoira/{id: u32}) => {
                handle_ugoira(&client, id)
            },
            (GET) (/rss) => {
                handle_rss(&client, &request)
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
                    rouille::Response::redirect_301(destination)
                } else if url == "member_illust.php" {
                    let id = match request.get_param("illust_id") {
                        Some(id) => id,
                        None => return render_error(401, "No illust id"),
                    };
                    let destination = format!("/artworks/{}", id);
                    rouille::Response::redirect_301(destination)
                } else {
                    render_error(404, "Endpoint not found!")
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

fn handle_ugoira(client: &ureq::Agent, id: u32) -> rouille::Response {
    let meta = match fetch_ugoira_meta(client, id) {
        Ok(meta) => meta,
        Err(_) => {
            println!("Error fetching ugoira meta: {}", id);
            return rouille::Response::empty_404();
        }
    };
    let ugoira = client
        .get(&meta.original_src)
        .set("Referer", "https://pixiv.net/")
        .set(
            "Cookie",
            &std::env::args().nth(1).expect("PIXIV_COOKIE must be set"),
        )
        .call()
        .unwrap();

    let reader: Box<dyn Read + Send> = Box::new(ugoira.into_reader());
    let reader = BufReader::with_capacity(0x4000, reader);

    struct Opaque<'a> {
        reader: BufReader<Box<dyn Read + Send>>,
        file: Option<zip::read::ZipFile<'a>>,
        writer: Cursor<Vec<u8>>,
    }
    let mut opaque = Opaque {
        reader,
        file: None,
        writer: Cursor::new(Vec::with_capacity(0x100000)),
    };

    unsafe extern "C" fn read(opaque: *mut libc::c_void, ptr: *mut u8, sz: i32) -> i32 {
        // println!("read: {}", sz);
        let opaque = opaque as *mut Opaque<'_>;
        let slice = std::slice::from_raw_parts_mut(ptr, sz as usize);
        let file = (*opaque).file.as_mut().unwrap();
        file.read(slice).unwrap() as i32
    }
    unsafe extern "C" fn next(opaque: *mut libc::c_void) {
        // println!("next");
        let opaque = opaque as *mut Opaque<'_>;
        let reader = &mut (*opaque).reader;
        (*opaque).file = Some(
            zip::read::read_zipfile_from_stream(reader)
                .unwrap()
                .unwrap(),
        );
    }

    unsafe extern "C" fn write(opaque: *mut libc::c_void, ptr: *mut u8, sz: i32) -> i32 {
        // println!("write: {}", sz);
        let opaque = opaque as *mut Opaque<'_>;
        let slice = std::slice::from_raw_parts(ptr, sz as usize);
        (*opaque).writer.write_all(slice).unwrap();
        sz
    }
    unsafe extern "C" fn seek(opaque: *mut libc::c_void, offset: i64, whence: i32) -> i64 {
        // println!("seek: {} {}", offset, whence);
        let opaque = opaque as *mut Opaque<'_>;
        let position = match whence {
            0 => std::io::SeekFrom::Start(offset as u64),
            1 => std::io::SeekFrom::Current(offset),
            2 => std::io::SeekFrom::End(offset),
            _ => panic!("invalid whence"),
        };
        (*opaque).writer.seek(position).unwrap() as i64
    }

    let start = std::time::Instant::now();

    extern "C" {
        fn convert(
            opaque: *mut libc::c_void,
            read: unsafe extern "C" fn(*mut libc::c_void, *mut u8, i32) -> i32,
            next: unsafe extern "C" fn(*mut libc::c_void),
            write: unsafe extern "C" fn(*mut libc::c_void, *mut u8, i32) -> i32,
            seek: unsafe extern "C" fn(*mut libc::c_void, i64, i32) -> i64,
            frames: *const UgoiraFrame,
            frame_count: usize,
        ) -> i32;
    }
    let ret = unsafe {
        convert(
            &mut opaque as *mut Opaque<'_> as *mut libc::c_void,
            read,
            next,
            write,
            seek,
            meta.frames.as_ptr(),
            meta.frames.len(),
        )
    };
    let duration = start.elapsed();
    println!("{:?}", duration);

    if ret != 0 {
        return rouille::Response {
            status_code: 500,
            headers: vec![],
            data: rouille::ResponseBody::empty(),
            upgrade: None,
        };
    }

    rouille::Response::from_data("video/mp4", opaque.writer.into_inner())
        .with_public_cache(365 * 24 * 60 * 60)
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

    fn make_option(name: &str, value: &str, mode: &str) -> maud::Markup {
        html! {
            @if mode == value {
                option value=(&value) selected { (&name) }
            } @else {
                option value=(&value) { (&name) }
            }
        }
    }

    let docs = document! {
        &tag,
        html! {
            h1 { (&tag) }
            (&search.illust_manga.total)
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
            (render_list(&search.illust_manga.data))
            @if search.illust_manga.total > 60 {
                @let format = format!("/tags/{}/artworks?order={}&mode={}&s_mode={}&p=", tag, order, mode, search_mode);
                (render_nav(page, search.illust_manga.total, &format))
            }
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
                        (date.unwrap().format("%Y-%m-%d %H:%M:%S -"))
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
            ul.illust__images {
                @match artwork.illust_type {
                    2 => {
                        @let src = format!("/ugoira/{}", id);
                        li {
                            video poster=(&image) src=(&src) autoplay="" loop="" muted="" {}
                        }
                    },
                    _ => @for url in std::iter::once(image.clone())
                        .chain(
                            (1..artwork.page_count).map(|i|
                                image.clone().replace("_p0.", &format!("_p{}.", i))
                            )
                        ) {
                        li { img src=(&url) alt=(&artwork.alt); }
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

    let image = util::image_to_proxy(&user.image_big);

    let doc = document! {
        &user.name,
        html! {
            header.author {
                img.logo src=(&image) alt=(&user.name) width="170";
                h1 { (&user.name) }
                p { (PreEscaped(&user.comment_html)) }
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

fn handle_rss(client: &ureq::Agent, request: &rouille::Request) -> rouille::Response {
    let query_type = request.get_param("type").unwrap_or("search".to_string());
    let query_words = request.get_param("q").unwrap_or("".to_string());
    let query_mode = request.get_param("mode").unwrap_or("all".to_string());
    let query_search_mode = request
        .get_param("s_mode")
        .unwrap_or("s_tag_full".to_string());

    let page = match query_type.as_str() {
        "author" => {
            let ids = match fetch_user_illust_ids(client, &query_words) {
                Ok(ids) => ids,
                Err(_) => {
                    return rouille::Response::empty_400();
                }
            };
            match fetch_user_illustrations(client, &query_words, &ids) {
                Ok(elements) => elements,
                Err(_) => {
                    return rouille::Response::empty_400();
                }
            }
        }
        "search" | _ => {
            let search = match fetch_search(
                client,
                &query_words,
                "date_d",
                &query_mode,
                "1",
                &query_search_mode,
            ) {
                Ok(search) => search,
                Err(_) => {
                    return rouille::Response::empty_400();
                }
            };
            search.illust_manga.data
        }
    };

    let items: Vec<rss::Item> = page
        .iter()
        .map(|s| {
            let link = format!("{}/artworks/{}", BASE_URL, s.id);
            let guid = rss::GuidBuilder::default()
                .value(link.clone())
                .permalink(true)
                .build();
            let date = chrono::DateTime::parse_from_rfc3339(&s.update_date);
            let description = match date {
                Ok(date) => {
                    let img_base = format!(
                        "{}/imageproxy/img-master/img/{}/{}",
                        BASE_URL,
                        date.format("%Y/%m/%d/%H/%M/%S"),
                        s.id
                    );
                    html!(
                        h1 { (&s.title) }
                        p { (date.format("%Y-%m-%d %H:%M:%S")) }
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
                        img src=(&s.url);
                    )
                }
            };
            rss::ItemBuilder::default()
                .title(Some(s.title.clone()))
                .link(Some(link.clone()))
                .guid(Some(guid))
                .description(Some(description.into_string()))
                .author(Some(s.user_name.clone()))
                .pub_date(Some(s.create_date.clone()))
                .build()
        })
        .collect();

    let link = format!("{}{}", BASE_URL, request.raw_url());

    let content = rss::ChannelBuilder::default()
        .title(query_words)
        .link(link)
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
        ul.search {
            @for artwork in list {
                @let link = format!("/en/artworks/{}", artwork.id);
                @let img = util::image_to_proxy(&artwork.url);
                li {
                    a href=(&link) {
                        @if artwork.r18 == 1 {
                            div.search__hover.search__warn { "R-18" }
                        }
                        @if artwork.page_count > 1 {
                            div.search__hover.search__count {
                                svg viewBox="0 0 10 10" {
                                    path d=(SVG_PAGE_PATH);
                                }
                                (artwork.page_count)
                            }
                        }
                        img src=(&img) width="250" height="250";
                    }
                    a href=(&link) { (&artwork.title) }
                }
            }
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
