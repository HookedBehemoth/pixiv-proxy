use actix_web::{get, HttpResponse};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    favicon
}

const FAVICON: &[u8] = include_bytes!("../../static/favicon.ico");

#[get("/favicon.ico")]
async fn favicon() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/x-icon")
        .body(FAVICON)
}
