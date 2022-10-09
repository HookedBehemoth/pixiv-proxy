use maud::html;

use crate::api::search::{SearchMode, SearchOrder, SearchRating};

impl maud::Render for SearchOrder {
    fn render(&self) -> maud::Markup {
        maud::html! { (self.as_str()) }
    }
}

impl maud::Render for SearchMode {
    fn render(&self) -> maud::Markup {
        maud::html! { (self.as_str()) }
    }
}

impl maud::Render for SearchRating {
    fn render(&self) -> maud::Markup {
        maud::html! { (self.as_str()) }
    }
}

pub fn render_options(
    tag: &str,
    rating: SearchRating,
    order: SearchOrder,
    mode: SearchMode,
) -> maud::Markup {
    fn make_option<T>(name: &str, value: T, mode: T) -> maud::Markup
    where
        T: PartialEq + maud::Render,
    {
        html! {
            @if mode == value {
                option value=(value) selected { (name) }
            } @else {
                option value=(value) { (name) }
            }
        }
    }

    html! {
        form action="/search" method="get" {
            input type="text" name="q" placeholder="Keywords..." value=(&tag) required;
            select name="mode" {
                (make_option("All", SearchRating::All, rating));
                (make_option("Safe", SearchRating::Safe, rating));
                (make_option("R-18", SearchRating::Adult, rating));
            }
            select name="order" {
                (make_option("By Upload Date (Newest)", SearchOrder::DateDescending, order));
                (make_option("By Upload Date (Oldest)", SearchOrder::DateAscending, order));
                (make_option("By Popularity (All)", SearchOrder::Popular, order));
                (make_option("By Popularity (Male)", SearchOrder::PopularMale, order));
                (make_option("By Popularity (Female)", SearchOrder::PopularFemale, order));
            }
            select name="s_mode" {
                (make_option("Tags (perfect match)", SearchMode::TagsPerfect, mode));
                (make_option("Tags (partial match)", SearchMode::TagsPartial, mode));
                (make_option("Title, Caption", SearchMode::TitleCaption, mode));
            }
            button type="submit" { "Search" }
        }
    }
}
