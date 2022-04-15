pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}
pub fn image_to_proxy(image: &str) -> String {
    image.replace("https://i.pximg.net/", "/imageproxy/")
}
