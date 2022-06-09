use std::fmt::Display;

use actix_web::error;
use maud::html;

use crate::api::error::ApiError;

use super::document::document;

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::External(code, message) => {
                f.write_fmt(format_args!("{}: {}", *code, message))
            }
            ApiError::Internal(message) => f.write_fmt(format_args!("{}", message)),
        }
    }
}

impl error::ResponseError for ApiError {}

impl maud::Render for ApiError {
    fn render(&self) -> maud::Markup {
        match self {
            ApiError::External(code, message) => render_error(*code, message),
            ApiError::Internal(message) => render_error(500, message),
        }
    }
}

pub fn render_error(code: u16, message: &str) -> maud::Markup {
    let title = format!("{}", code);

    document(
        &title,
        html! {
            h1 { (&title) }
            p { (message) }
            p { a href="/" { "Go home" } }
        },
        None,
    )
}
