use std::borrow::Cow;
use rouille::{Response, ResponseBody};

pub fn handle_imageproxy(client: &ureq::Agent, path: &str) -> Response {
    const OFFSET: usize = "/imageproxy/".len();
    let url = format!("https://i.pximg.net/{}", &path[OFFSET..]);
    let response = client.get(&url).call().unwrap();

    let headers = response
        .headers_names()
        .iter()
        .filter(|&s| s != "cookies")
        .map(|s| {
            (
                Cow::from(s.clone()),
                Cow::from(response.header(s).unwrap_or("").to_string()),
            )
        })
        .collect();

    let reader = match response
        .header("Content-Length")
        .map(|s| s.parse::<usize>())
    {
        Some(Ok(len)) => ResponseBody::from_reader_and_size(response.into_reader(), len),
        _ => ResponseBody::from_reader(response.into_reader()),
    };

    Response {
        status_code: 200,
        headers: headers,
        data: reader,
        upgrade: None,
    }
}
