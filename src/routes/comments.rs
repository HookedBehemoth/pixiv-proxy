use actix_web::{get, web, Result};
use maud::{html, Markup};
use serde::Deserialize;

use crate::api::comments::{fetch_comments, fetch_replies};

pub fn routes() -> impl actix_web::dev::HttpServiceFactory {
    (comments, replies)
}

#[derive(Deserialize)]
struct CommentsRequest {
    #[serde(default)]
    offset: u32,
    limit: Option<u32>,
}

#[get("/comments/{id}")]
async fn comments(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
    query: web::Query<CommentsRequest>,
) -> Result<Markup> {
    let id = *id;
    let offset = query.offset;
    let limit = query.limit.unwrap_or(100);

    let roots = fetch_comments(&client, id, offset, limit)
        .await
        .map_err(actix_web::error::ErrorNotFound)?;

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

    Ok(doc)
}

#[derive(Deserialize)]
struct RepliesRequest {
    page: Option<u32>,
}

#[get("/replies/{id}")]
async fn replies(
    client: web::Data<awc::Client>,
    id: web::Path<u64>,
    query: web::Query<RepliesRequest>,
) -> Result<Markup> {
    let id = *id;
    let page = query.page.unwrap_or(1);

    let replies = fetch_replies(&client, id, page)
        .await
        .map_err(actix_web::error::ErrorNotFound)?;

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

    Ok(doc)
}
