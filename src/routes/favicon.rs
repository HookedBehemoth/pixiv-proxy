const FAVICON: &[u8] = include_bytes!("../../static/favicon.ico");

pub fn favicon() -> rouille::Response {
    rouille::Response::from_data("image/x-icon", FAVICON)
}
