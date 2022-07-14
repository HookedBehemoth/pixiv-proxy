use crate::render::document::document;
use actix_web::{get, Responder};
use maud::html;

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    about
}

#[get("/about")]
async fn about() -> impl Responder {
    document(
        "About",
        html! {
            h1 { "About" }
            p { "This is a simple Pixiv API client written in Rust." }
            p { "The source code is available on " a href="https://github.com/HookedBehemoth/pixiv-proxy" { "GitHub" } "." }

            h2 { "Contact" }
            p { "If you have any questions, feel free to contact me at " a href = "mailto:info@cunnycon.org" { "info@cunnycon.org" } "." }

            h2 { "License" }
            p { "This project is licensed under the " a href="https://www.gnu.org/licenses/licenses.html#AGPL" { "GNU Affero General Public License" } "." }
        },
        None,
    )
}
