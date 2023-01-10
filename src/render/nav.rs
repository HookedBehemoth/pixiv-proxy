use maud::html;

pub fn render_nav(current_page: u32, count: usize, limit: usize, template: &str) -> maud::Markup {
    html! {
        @if count > limit {
            nav {
                @let min = 1;
                @let max = count / limit + 1;
                @let nav_start = std::cmp::max(min as i32, current_page as i32 - 3);
                @let nav_end = std::cmp::min(max as i32, nav_start + 7);
                a href=(format!("{}{}", template, min)) { "<<" }
                @for page in nav_start..=nav_end {
                    @if page as u32 == current_page {
                        span { (page) }
                    } @else {
                        @let link = format!("{}{}", template, page);
                        a href=(&link) { (page) }
                    }
                }
                a href=(format!("{}{}", template, max - 1)) { ">>" }
            }
        }
    }
}
