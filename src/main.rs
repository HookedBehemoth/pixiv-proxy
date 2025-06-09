mod api;
mod render;
mod routes;
mod util;

use api::error::ApiError;
use routes::*;
use ureq::{
    http::{self, HeaderValue},
    middleware::MiddlewareNext,
    Body, SendBody,
};

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:103.0) Gecko/20100101 Firefox/103.0";

fn main() -> std::io::Result<()> {
    let mut pargs = pico_args::Arguments::from_env();
    let port: u32 = pargs.value_from_str("--port").unwrap_or(8000);
    let listen_address = format!("http://localhost:{}", port);
    let host: String = pargs
        .value_from_str("--host")
        .unwrap_or_else(|_| listen_address.clone());
    let cookie: String = match pargs.value_from_str("--cookie") {
        Ok(cookie) => cookie,
        Err(_) => {
            println!("No cookie set. Fetching generic one.");
            println!("Keep in mind that this will offer very limited functionality.");

            let client = ureq::Agent::new_with_defaults();
            let res = client
                .get("https://www.pixiv.net/en/")
                .call()
                .expect("Can't contact Pixiv to fetch default header!");

            let cookie = res
                .headers()
                .iter()
                .filter(|h| h.0 == "set-cookie")
                .filter_map(|h| h.1.to_str().ok())
                .find(|h| h.starts_with("PHPSESSID"))
                .expect("No cookie obtained!")
                .to_owned();

            let length = cookie.find("; ").unwrap();
            cookie[..length].to_owned()
        }
    };

    println!("Listening on {}", listen_address);
    println!("Cookies: {}", cookie);

    /* Build HTTP client */
    let client = {
        /* Load tls certificate */
        let tls_config = ureq::tls::TlsConfig::builder().build();

        /* Add default headers */
        struct PixivDefaultHeaders {
            referer: String,
            cookie: String,
        }

        impl ureq::middleware::Middleware for PixivDefaultHeaders {
            fn handle(
                &self,
                mut request: http::Request<SendBody>,
                next: MiddlewareNext,
            ) -> Result<http::Response<Body>, ureq::Error> {
                let headers = request.headers_mut();
                headers.append("Referer", HeaderValue::from_str(&self.referer).unwrap());
                headers.append("Cookie", HeaderValue::from_str(&self.cookie).unwrap());
                next.handle(request)
            }
        }

        let middleware: PixivDefaultHeaders = PixivDefaultHeaders {
            referer: "https://pixiv.net/".to_string(),
            cookie,
        };

        /* Build https client */
        let config = ureq::Agent::config_builder()
            .tls_config(tls_config)
            .user_agent(USER_AGENT)
            .middleware(middleware)
            .max_redirects(0)
            .build();

        ureq::Agent::new_with_config(config)
    };

    /* Build RSS config */
    let rss_config = routes::rss::RssConfig { host };

    let address = format!("0.0.0.0:{}", port);
    rouille::start_server(&address, move |request| {
        let result = rouille::router!(request,
            /* Front page */
            (GET) ["/"] => { ranking::ranking(&client, request) },

            /* Search */
            (GET) ["/tags/{tag}", tag: String] => { search::tags(&client, &tag, request) },
            (GET) ["/tags/{tag}/artworks", tag: String] => { search::tags(&client, &tag, request) },
            (GET) ["/search"] => { search::query_search(&client, request) },

            /* Scrolling image view */
            (GET) ["/scroll"] => { scroll::scroll(&client, request) },

            /* Users */
            (GET) ["/users/{id}", id: u64] => { users::artworks(&client, id, request) },
            (GET) ["/users/{id}/artworks", id: u64] => { users::artworks(&client, id, request) },
            (GET) ["/users/{id}/bookmarks/artworks", id: u64] => { users::bookmarks(&client, id, request) },

            /* Artworks */
            (GET) ["/artworks/{id}", id: u64] => { artworks::artwork(&client, id) },

            /* Comments */
            (GET) ["/comments/{id}", id: u64] => { comments::comments(&client, id, request) },
            (GET) ["/replies/{id}", id: u64] => { comments::replies(&client, id, request) },

            /* Jump pads */
            (GET) ["/jump.php"] => { redirect::jump(request) },
            (GET) ["/member_illust.php"] => { redirect::legacy_illust(request) },
            (GET) ["/fanbox/creator/{id}", id: u64] => { redirect::fanbox(&client, id) },

            /* Sketch */
            (GET) ["/sketch"] => { sketch::sketch_public(&client) },
            (GET) ["/sketch/tags/{tag}", tag: String] => { sketch::sketch_tags(&client, &tag) },
            (GET) ["/sketch/users/{id}", id: u64] => { sketch::sketch_user(&client, id) },
            (GET) ["/sketch/items/{id}", id: u64] => { sketch::sketch_item(&client, id) },
            (GET) ["/sketch/lives"] => { sketch::sketch_lives(&client) },
            (GET) ["/sketch/impressions/{id}", id: u64] => { sketch::sketch_user(&client, id) },

            /* Ugoira */
            (GET) ["/ugoira/{id}", id: u64] => { ugoira::ugoira(&client, id) },

            /* RSS */
            (GET) ["/rss"] => { rss::rss(&client, request, &rss_config) },

            /* Image proxy */
            (GET) ["/stamp/{id}", id: u32] => { imageproxy::stamp(&client, id, request) },

            /* Stylesheet */
            (GET) ["/stylesheet.css"] => { Ok(css::style_sheet()) },
            (GET) ["/favicon.ico"] => { Ok(favicon::favicon()) },

            /* Settings */
            (GET) ["/settings"] => { Ok(settings::index(request)) },
            (POST) ["/settings/blocked/add"] => { Ok(settings::blocked_users_add(request)) },
            (POST) ["/settings/blocked/del"] => { Ok(settings::blocked_users_del(request)) },

            /* About */
            (GET) ["/about"] => { Ok(about::about()) },

            _ => {
                let path = request.url();
                if let Some(path) = path.strip_prefix("/en") {
                    Ok(rouille::Response::redirect_301(path.to_owned()))
                } else if let Some(response) = imageproxy::imageproxy(&client, &path, request) {
                    response
                } else if let Some(response) = imageproxy::s_imageproxy(&client, &path, request) {
                    response
                } else if let Some(response) = imageproxy::spix_imageproxy(&client, &path, request) {
                    response
                } else if let Some(response) = imageproxy::spxi_imageproxy(&client, &path, request) {
                    response
                } else {
                    Err(ApiError::External(404, "Not Found".into()))
                }
            }
        );

        match result {
            Ok(response) => response,
            Err(error) => {
                let page = match error {
                    ApiError::Internal(message) => render::error::render_error(500, &message),
                    ApiError::External(code, message) => {
                        render::error::render_error(code, &message)
                    }
                };
                rouille::Response::html(page)
            }
        }
    })
}
