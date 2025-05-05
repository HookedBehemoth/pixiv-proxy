use std::{collections::HashSet, io::Read};

use crate::render::document::document;
use maud::html;
use rouille::input::cookies;

const SEPERATOR: &str = "%7C";
const BLOCKED_COOKIE: &str = "blocked_users";

pub fn get_blocked_userids(request: &rouille::Request) -> HashSet<u64> {
    let Some((_, v)) = cookies(request).find(|&(k, _)| k == BLOCKED_COOKIE) else {
        return HashSet::new();
    };

    v.split(SEPERATOR)
        .map(|id| id.parse::<u64>())
        .filter(|p| p.is_ok())
        .fold(HashSet::<u64>::new(), |mut set, user| {
            set.insert(user.unwrap());
            set
        })
}

fn format_blocked_userids(users: &HashSet<u64>) -> String {
    users.iter().fold(String::new(), |set, user| {
        if set.is_empty() {
            format!("{user}")
        } else {
            format!("{set}|{user}")
        }
    })
}

pub fn set_blocked_userids(response: &mut rouille::Response, users: &HashSet<u64>) {
    let cookie_value = users.iter().fold(String::new(), |set, user| {
        if set.is_empty() {
            format!("{user}")
        } else {
            format!("{set}%7C{user}")
        }
    });
    response.headers.push((
        "Set-Cookie".into(),
        format!("{BLOCKED_COOKIE}={cookie_value}; Path=/").into(),
    ));
}

pub fn index(request: &rouille::Request) -> rouille::Response {
    let blocked_users = get_blocked_userids(request);

    let document = document(
        "Settings",
        html! {
            h1 { "Settings" }
            p { "All settings are stored in Cookies that are stored in your browser." }
            p { "No data is kept on the server after process your requests." }

            h2 { "Blocked Users" }
            p {
                "You can either select the \"Block\" Button on a User-Profile or import a list off the format \"12345|23456|34567\" here. The name is currently only used for this settings page."
            }
            div {
                form action="/settings/blocked/add" method="POST" {
                    input type="text" name="bulk" { }
                    input type="submit" { }
                }
            }
            form action="/settings/blocked/del" method="POST" {
                ul {
                    @for user in &blocked_users {
                        li {
                            a href=(format!("/users/{user}")) {
                                (user)
                            }
                            input type="submit" name=(user) value="Unblock" {}
                        }
                    }
                }
            }
            @if !blocked_users.is_empty() {
                p {
                    "Your current block list goes as follows. You can back it up and import it later on."
                }
                p {
                    (format_blocked_userids(&blocked_users))
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

    let Some((key, value)) = user.split_once('=') else {
        return redirect;
    };

    let mut blocked = get_blocked_userids(request);

    if key == "bulk" {
        for id in value
            .split(SEPERATOR)
            .map(|v| v.parse::<u64>())
            .filter(|v| v.is_ok())
        {
            blocked.insert(id.unwrap());
        }
    } else if let Ok(id) = key.parse::<u64>() {
        blocked.insert(id);
    } else {
        return redirect;
    }

    set_blocked_userids(&mut redirect, &blocked);

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

    let Some((key, value)) = user.split_once('=') else {
        return redirect;
    };

    let mut blocked = get_blocked_userids(request);

    if key == "bulk" {
        for id in value
            .split(SEPERATOR)
            .map(|v| v.parse::<u64>())
            .filter(|v| v.is_ok())
        {
            blocked.remove(&id.unwrap());
        }
    } else if let Ok(id) = key.parse::<u64>() {
        blocked.remove(&id);
    } else {
        return redirect;
    }

    set_blocked_userids(&mut redirect, &blocked);

    redirect
}
