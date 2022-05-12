use chrono::{DateTime, Utc};
use maud::{html, Markup, Render};

pub struct DateTimeWrapper(pub DateTime<Utc>);
impl Render for DateTimeWrapper {
    fn render(&self) -> Markup {
        html! {
            (self.0.format("%Y-%m-%d %H:%M:%S"))
        }
    }
}
