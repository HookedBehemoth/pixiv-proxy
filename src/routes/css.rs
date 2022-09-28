const CSS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/main.css"));

pub fn style_sheet() -> rouille::Response {
    rouille::Response::from_data("text/css", CSS)
        .with_unique_header("Cache-Control", "public,max-age=31536000")
}
