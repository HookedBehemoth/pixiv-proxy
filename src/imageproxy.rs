use rouille::{Response, ResponseBody};
use std::borrow::Cow;

pub fn handle_imageproxy(client: &ureq::Agent, path: &str) -> Response {
    const OFFSET: usize = "/imageproxy/".len();
    let url = format!("https://i.pximg.net/{}", &path[OFFSET..]);

    proxy(client, &url)
}

pub fn handle_stamp(client: &ureq::Agent, id: u32) -> rouille::Response {
    let url = format!(
        "https://s.pximg.net/common/images/stamp/generated-stamps/{}_s.jpg?20180605",
        id
    );

    proxy(client, &url)
}

fn proxy(client: &ureq::Agent, url: &str) -> Response {
    let response = match client.get(url).call() {
        Ok(response) => response,
        Err(_) => return Response::empty_404(),
    };

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
        headers,
        data: reader,
        upgrade: None,
    }
}
