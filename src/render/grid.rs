use std::collections::HashSet;

use crate::{api::common::PixivSearchResult, render::svg, util};

use maud::html;

pub fn render_grid(list: &[PixivSearchResult], blocked_users: &HashSet<u64>, load_more: Option<maud::Markup>) -> maud::Markup {
    html! {
        svg style="display:none" {
            defs {
                (svg::page())
                (svg::play())
            }
        }
        ul.search {
            (render_grid_contents(list, blocked_users))
            @if let Some(load_more) = load_more {
                (load_more)
            }
        }
    }
}

pub fn render_grid_contents(list: &[PixivSearchResult], blocked_users: &HashSet<u64>) -> maud::Markup {
    html! {
        @for artwork in list.iter().filter(|a| !blocked_users.contains(&a.user_id)) {
            @let link = format!("/artworks/{}", artwork.id);
            @let link = if !artwork.is_masked { Some(&link) } else { None };
            @let img = util::image_square_to_master(&artwork.url);
            @let (width, height) = util::scale_by_aspect_ratio(artwork.width, artwork.height, 200, 400);
            li {
                a href=[link] {
                    @if artwork.r18 == 1 {
                        div.search__hover.search__warn { "R-18" }
                    }
                    @if artwork.page_count > 1 {
                        div.search__hover.search__count {
                            svg { use href="#page" {} }
                            (artwork.page_count)
                        }
                    }
                    @if artwork.illust_type == 2 {
                        svg.search__play { use href="#play" {} }
                    }
                    img src=(&img) width=(width) height=(height) alt=(&artwork.title);
                }
                a href=[link] { (&artwork.title) }
            }
        }
    }
}
