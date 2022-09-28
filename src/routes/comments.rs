use maud::html;

use crate::api::{
    comments::{fetch_comments, fetch_replies},
    error::ApiError,
};
use crate::get_param_or_num;

pub fn comments(
    client: &ureq::Agent,
    id: u64,
    query: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let offset = get_param_or_num!(query, "offset", 0);
    let limit = get_param_or_num!(query, "limit", 100);

    let roots = fetch_comments(&client, id, offset, limit)?;

    let document = html! {
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

    Ok(rouille::Response::html(document.into_string()))
}

pub fn replies(
    client: &ureq::Agent,
    id: u64,
    query: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let page = get_param_or_num!(query, "page", 1);

    let replies = fetch_replies(&client, id, page)?;

    let document = html! {
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

    Ok(rouille::Response::html(document.into_string()))
}
