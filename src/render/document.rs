use maud::html;

const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/main.css"));

pub fn document(title: &str, content: maud::Markup, head: Option<maud::Markup>) -> maud::Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { (title) }
                style { (CSS) }
                meta name="viewport" content="width=device-width, initial-scale=1";
                @if head.is_some() { (head.unwrap()) }
            }
            body {
                main { (content) }
                footer { div { a href="/" { "Home" } " - " a href="/about" { "About" } } }
            }
        }
    }
}
