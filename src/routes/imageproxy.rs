use actix_web::{error, get, web, Error, HttpRequest, HttpResponse};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    (imageproxy, s_imageproxy, spix_imageproxy, spxi_imageproxy, stamp)
}

macro_rules! make_proxy {
    ($name:ident, $path:expr, $dest:tt) => {
        #[get($path)]
        async fn $name(
            client: web::Data<awc::Client>,
            request: HttpRequest,
            path: web::Path<String>,
        ) -> Result<HttpResponse, Error> {
            let url = format!($dest, path);
        
            proxy(&client, request, &url).await
        }
    };
}

make_proxy!(imageproxy, "/imageproxy/{path:[^{}?]+}", "https://i.pximg.net/{}");
make_proxy!(s_imageproxy, "/simg/{path:[^{}?]+}", "https://s.pximg.net/{}");
make_proxy!(spix_imageproxy, "/spix/{path:[^{}?]+}", "https://img-sketch.pixiv.net/{}");
make_proxy!(spxi_imageproxy, "/spxi/{path:[^{}?]+}", "https://img-sketch.pximg.net/{}");

#[get("/stamp/{id}")]
async fn stamp(
    client: web::Data<awc::Client>,
    request: HttpRequest,
    id: web::Path<u32>,
) -> Result<HttpResponse, Error> {
    let url = format!(
        "https://s.pximg.net/common/images/stamp/generated-stamps/{}_s.jpg?20180605",
        id
    );

    proxy(&client, request, &url).await
}

/* Note: passing these to the client should be avoided */
const FORBIDDEN_CLIENT_HEADERS: &[&str] = &["connection", "cookies"];
const FORBIDDEN_SERVER_HEADERS: &[&str] =
    &["connection", "cookie", "user-agent", "host", "referer"];

async fn proxy(
    client: &awc::Client,
    request: HttpRequest,
    url: &str,
) -> Result<HttpResponse, Error> {
    let mut req = client.get(url);

    for header in request
        .headers()
        .iter()
        .filter(|(h, _)| !FORBIDDEN_SERVER_HEADERS.contains(&h.as_str()))
    {
        req = req.insert_header(header);
    }

    let res = req.send().await.map_err(error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());
    for header in res
        .headers()
        .iter()
        .filter(|(h, _)| !FORBIDDEN_CLIENT_HEADERS.contains(&h.as_str()))
    {
        client_resp.insert_header(header);
    }

    Ok(client_resp.streaming(res))
}
