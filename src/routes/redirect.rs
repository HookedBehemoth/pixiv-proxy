use actix_web::{get, web, HttpRequest, HttpResponse, Result};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    (jump, legacy_illust, fanbox)
}

#[get("/jump.php")]
async fn jump(path: HttpRequest) -> HttpResponse {
    let destination = path.query_string();
    let destination = percent_encoding::percent_decode_str(destination)
        .decode_utf8_lossy()
        .into_owned();

    HttpResponse::PermanentRedirect()
        .append_header(("Location", destination))
        .finish()
}

#[derive(serde::Deserialize)]
struct RedirectLegacy {
    illust_id: u64,
}

#[get("/member_illust.php")]
async fn legacy_illust(query: web::Query<RedirectLegacy>) -> HttpResponse {
    println!("legacy {}", query.illust_id);
    let destination = format!("/artworks/{}", query.illust_id);
    HttpResponse::PermanentRedirect()
        .append_header(("Location", destination))
        .finish()
}

#[get("/fanbox/creator/{id}")]
async fn fanbox(client: web::Data<awc::Client>, id: web::Path<u64>) -> Result<HttpResponse> {
    let url = format!("https://www.pixiv.net/fanbox/creator/{}", id);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(actix_web::error::ErrorNotFound)?;

    let location = response
        .headers()
        .get("Location")
        .ok_or_else(|| actix_web::error::ErrorNotFound(""))?;

    Ok(HttpResponse::PermanentRedirect()
        .append_header(("Location", location))
        .finish())
}
