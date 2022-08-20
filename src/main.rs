mod api;
mod render;
mod routes;
mod util;

use actix_web::{web, App, HttpServer};
use awc::{http::header, Client, Connector};

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:103.0) Gecko/20100101 Firefox/103.0";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

            let client = awc::Client::default();
            let res = client
                .get("https://www.pixiv.net/en/")
                .send()
                .await
                .expect("Can't contact Pixiv to fetch default header!");

            let cookie = res
                .headers()
                .get_all("set-cookie")
                .map(|c| c.to_str().unwrap())
                .find(|&c| c.starts_with("PHPSESSID"))
                .unwrap();

            let length = cookie.find("; ").unwrap();
            cookie[..length].to_owned()
        }
    };

    println!("Listening on {}", listen_address);
    println!("Cookies: {}", cookie);

    let address = format!("0.0.0.0:{}", port);
    HttpServer::new(move || {
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

        /* Build https client */
        let client = Client::builder()
            .add_default_header(("Referer", "https://pixiv.net/"))
            .add_default_header(("Cookie", cookie.as_str()))
            .add_default_header((header::USER_AGENT, USER_AGENT))
            .connector(Connector::new().rustls(tls_config))
            .finish();

        /* Build RSS config */
        let rss_config = routes::rss::RssConfig { host: host.clone() };

        App::new()
            .app_data(web::Data::new(client))
            .app_data(web::Data::new(rss_config))
            .service(routes::artworks::routes())
            .service(routes::search::routes())
            .service(routes::users::routes())
            .service(routes::imageproxy::routes())
            .service(routes::ugoira::routes())
            .service(routes::redirect::routes())
            .service(routes::ranking::routes())
            .service(routes::scroll::routes())
            .service(routes::comments::routes())
            .service(routes::about::routes())
            .service(routes::rss::routes())
            .service(routes::favicon::routes())
            .service(routes::sketch::routes())
            .service(routes::css::routes())
            .default_service(web::to(route_404))
    })
    .bind(&address)?
    .run()
    .await
}

async fn route_404() -> actix_web::HttpResponse {
    actix_web::HttpResponse::NotFound()
        .body(render::error::render_error(404, "Resource not found").into_string())
}
