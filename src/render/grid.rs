use crate::{api::common::PixivSearchResult, render::svg};

use maud::html;

pub fn render_grid(list: &[PixivSearchResult]) -> maud::Markup {
    html! {
        svg style="display:none" {
            defs {
                (svg::page())
                (svg::play())
            }
        }
        ul.search {
            @for artwork in list {
                @let link = format!("/artworks/{}", artwork.id);
                @let link = if !artwork.is_masked { Some(&link) } else { None };
                @let img = &artwork.url;
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
                        img src=(&img) width="200" height="200" alt=(&artwork.title);
                    }
                    a href=[link] { (&artwork.title) }
                }
            }
        }
    }
}
