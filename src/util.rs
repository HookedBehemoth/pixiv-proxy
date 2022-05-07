pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}
pub fn image_to_proxy(image: &str) -> String {
    image.replace("https://i.pximg.net/", "/imageproxy/")
}
pub fn scale_by_aspect_ratio(width: u32, height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    if width < max_width && height < max_height {
        return (width, height);
    }
    let ratio = width as f32 / height as f32;
    let max_ratio = max_width as f32 / max_height as f32;
    if ratio > max_ratio {
        (max_width, f32::ceil(max_width as f32 / ratio) as u32)
    } else {
        (f32::ceil(max_height as f32 * ratio) as u32, max_height)
    }
}
