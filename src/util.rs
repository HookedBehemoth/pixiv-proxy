pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}
pub fn scale_by_aspect_ratio(
    width: u32,
    height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32) {
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

#[macro_export]
macro_rules! get_param_or_str {
    ($request:expr, $name:expr, $default:expr) => {{
        let option = $request.get_param($name);
        match option {
            Some(value) => std::borrow::Cow::from(value),
            None => std::borrow::Cow::from($default),
        }
    }};
}
#[macro_export]
macro_rules! get_param_or_num {
    ($request:expr, $name:expr, $default:expr) => {{
        let option = $request.get_param($name);
        match option {
            Some(value) => value.parse::<u32>().unwrap_or($default),
            None => $default,
        }
    }};
}
#[macro_export]
macro_rules! get_param_or_enum {
    ($request:expr, $name:expr, $enum:ty, $default:expr) => {{
        let option = $request.get_param($name);
        match option {
            Some(value) => <$enum>::from_str(&value).unwrap_or($default),
            None => $default,
        }
    }};
}
