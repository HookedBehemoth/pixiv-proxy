use actix_web::{get, HttpResponse};

const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/main.css"));

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    style_sheet
}

#[get("stylesheet.css")]
async fn style_sheet() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css")
        .append_header(("Cache-Control", "public,max-age=31536000"))
        .body(CSS)
}
