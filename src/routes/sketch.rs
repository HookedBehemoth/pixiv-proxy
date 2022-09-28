use crate::{
    api::{
        error::ApiError,
        sketch::{
            fetch_feedbacks, fetch_item, fetch_latest_user_posts, fetch_lives, fetch_public_wall,
            fetch_tag_wall, fetch_user,
        },
    },
    render::document::document,
};

use maud::html;

pub fn sketch_public(client: &ureq::Agent) -> Result<rouille::Response, ApiError> {
    let wall = fetch_public_wall(&client, None, None)?.data;

    let document = document(
        "Sketch",
        html! {
            ul.sketch_wall {
                @for item in &wall.items {
                    li { (item) }
                }
            }
            // span style="white-space:pre-wrap" {(serde_json::to_string_pretty(&wall).unwrap())}
        },
        None,
    );

    Ok(rouille::Response::html(document.into_string()))
}

pub fn sketch_tags(
    client: &ureq::Agent,
    tag: &str,
) -> Result<rouille::Response, ApiError> {
    let wall = fetch_tag_wall(&client, &tag)?.data;

    let document = document(
        "Sketch",
        html! {
            ul.sketch_wall {
                @for item in &wall.items {
                    li { (item) }
                }
            }
            // span style="white-space:pre-wrap" {(serde_json::to_string_pretty(&wall).unwrap())}
        },
        None,
    );

    Ok(rouille::Response::html(document.into_string()))
}

pub fn sketch_user(
    client: &ureq::Agent,
    id: u64,
) -> Result<rouille::Response, ApiError> {
    let user = fetch_user(&client, id)?;

    // TODO
    let ajax = serde_json::to_string_pretty(&user).unwrap();

    Ok(rouille::Response::html(ajax))
}

pub fn sketch_item(
    client: &ureq::Agent,
    id: u64,
) -> Result<rouille::Response, ApiError> {
    let item = fetch_item(&client, id)?;

    let ajax = serde_json::to_string_pretty(&item).unwrap();

    Ok(rouille::Response::html(ajax))
}

pub fn sketch_lives(client: &ureq::Agent) -> Result<rouille::Response, ApiError> {
    let lives = fetch_lives(&client, 20, "audience_count")?;

    let ajax = serde_json::to_string_pretty(&lives).unwrap();

    Ok(rouille::Response::html(ajax))
}

pub fn sketch_impressions(
    client: &ureq::Agent,
    id: u64,
) -> Result<rouille::Response, ApiError> {
    let impressions = fetch_feedbacks(&client, id)?;

    let ajax = serde_json::to_string_pretty(&impressions).unwrap();

    Ok(rouille::Response::html(ajax))
}
