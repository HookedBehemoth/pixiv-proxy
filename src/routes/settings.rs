use std::{collections::HashSet, io::Read};

use crate::render::document::document;
use maud::html;
use rouille::input::cookies;

pub struct BlockedUser {
    /// Display name (only useful for the settings page)
    pub name: String,
    pub id: u64,
}

const SEPERATOR: char = '|';
const BLOCKED_COOKIE: &str = "blocked_users";

pub fn get_blocked_users(request: &rouille::Request) -> Vec<BlockedUser> {
    cookies(request)
        .find(|&(k, _)| k == BLOCKED_COOKIE)
        .map(|(_, v)| v)
        .map(|v| {
            v.split(SEPERATOR)
                .map(|v| v.split_once('='))
                .filter(|v| v.is_some())
                .map(|v| v.unwrap())
                .map(|(name, id)| BlockedUser {
                    name: name.to_string(),
                    id: id.parse().unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_blocked_userids(request: &rouille::Request) -> HashSet<u64> {
    let blocked_users = get_blocked_users(request);
    blocked_users
        .iter()
        .fold(HashSet::<u64>::new(), |mut set, user| {
            set.insert(user.id);
            set
        })
}

pub fn index(request: &rouille::Request) -> rouille::Response {
    let blocked_users = get_blocked_users(request);

    let document = document(
        "Settings",
        html! {
            h1 { "Settings" }
            p { "All settings are stored in Cookies that are stored in your browser." }
            p { "No data is kept on the server after process your requests." }

            h2 { "Blocked Users" }
            p {
                "You can either select the \"Block\" Button on a User-Profile or import a list off the format \"[user name]=[user id];...\" here. The name is currently only used for this settings page."
            }
            div {
                form action="/settings/blocked/add" method="POST" {
                    input type="text" name="bulk" { }
                    input type="submit" { }
                }
            }
            form action="/settings/blocked/del" method="POST" {
                ul {
                    @for user in blocked_users {
                        li {
                            a href=(format!("/users/{}", user.id)) {
                                (user.name)
                                " - "
                                (user.id)
                                " - "
                                input type="submit" name=(user.name) value=(user.id) {}
                            }
                        }
                    }
                }
            }
        },
        None,
    );

    rouille::Response::html(document.into_string())
}

pub fn blocked_users_add(request: &rouille::Request) -> rouille::Response {
    let redirect = request.header("Referer").unwrap_or("/settings").to_string();
    let mut redirect = rouille::Response::redirect_303(redirect);

    let Some(mut data) = request.data() else {
        return redirect;
    };

    let mut user = String::new();
    let Ok(_) = data.read_to_string(&mut user) else {
        return redirect;
    };

    let user = user.replace("bulk=", "");

    let cookie_value = cookies(request)
        .find(|&(k, _)| k == BLOCKED_COOKIE)
        .map(|(_, v)| v)
        .map(|old| format!("{old}{SEPERATOR}{user}"))
        .unwrap_or(user);

    redirect.headers.push((
        "Set-Cookie".into(),
        format!("{BLOCKED_COOKIE}={cookie_value}; Path=/").into(),
    ));

    redirect
}

pub fn blocked_users_del(request: &rouille::Request) -> rouille::Response {
    let redirect = request.header("Referer").unwrap_or("/settings").to_string();
    let mut redirect = rouille::Response::redirect_303(redirect);

    let Some(mut data) = request.data() else {
        return redirect;
    };

    let mut user = String::new();
    let Ok(_) = data.read_to_string(&mut user) else {
        return redirect;
    };

    let Some(cookie_value) = cookies(request)
        .find(|&(k, _)| k == BLOCKED_COOKIE)
        .map(|(_, v)| v)
    else {
        return redirect;
    };

    let cookie_value = cookie_value.replace(user.as_str(), "");

    redirect.headers.push((
        "Set-Cookie".into(),
        format!("{BLOCKED_COOKIE}={cookie_value}; Path=/").into(),
    ));

    redirect
}
