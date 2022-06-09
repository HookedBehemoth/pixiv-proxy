use maud::{html, Markup};

pub fn page() -> Markup {
    html! {
        symbol id="page" viewBox="0 0 10 10" { path d="M8 3c.55 0 1 .45 1 1v5c0 .55-.45 1-1 1H3c-.55 0-1-.45-1-1h4c1.1 0 2-.9 2-2V3zM1 1h5c.55 0 1 .45 1 1v5c0 .55-.45 1-1 1H1c-.55 0-1-.45-1-1V2c0-.55.45-1 1-1z" {} }
    }
}

pub fn play() -> Markup {
    html! {
        symbol id="play" viewBox="0 0 100 100" {
            circle fill="#1f1f1fd0" cx="50" cy="50" r="50" {}
            path fill="#fff" d="M57.5 37C35 24 35 24 35 50s0 26 22.5 13 22.5-13 0-26" {}
        }
    }
}

pub fn like() -> Markup {
    html! {
        svg viewBox="0 0 12 12" { path d="M2 6a2 2 0 110-4 2 2 0 010 4zm8 0a2 2 0 110-4 2 2 0 010 4zM2.11 8.89a1 1 0 011.415-1.415 3.5 3.5 0 004.95 0 1 1 0 011.414 1.414 5.5 5.5 0 01-7.778 0z" fill="currentColor" {} }
    }
}

pub fn heart() -> Markup {
    html! {
        svg viewBox="6 7 20 19" { path d="M16 11C15 9 13 7.5 11 7.5a5 5 0 0 0-5 5c0 5 3.25 9.25 9.75 13a.5.5 0 0 0 .5 0C22.75 21.75 26 17.5 26 12.5a5 5 0 0 0-5-5c-2 0-4 1.5-5 3.5z" fill="currentColor" {} }
    }
}

pub fn eye() -> Markup {
    html! {
        svg viewBox="0 0 14 12" {
            path d="M0 6c2-3.333 4.333-5 7-5s5 1.667 7 5c-2 3.333-4.333 5-7 5S2 9.333 0 6z" fill="currentColor" {}
            path d="M7 8.5a2.5 2.5 0 110-5 2.5 2.5 0 010 5zm0-1a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" fill="black" {}
        }
    }
}
