use rouille::{Request, Response};

pub fn redirect_jump(request: &Request) -> Response {
    let destination = request.raw_query_string().as_bytes();
    let destination = percent_encoding::percent_decode(destination)
        .decode_utf8_lossy()
        .into_owned();
    Response::redirect_301(destination)
}

pub fn redirect_legacy_illust(request: &Request) -> Response {
    let id = match request.get_param("illust_id") {
        Some(id) => id,
        None => return Response::redirect_301("/"),
    };
    let destination = format!("/artworks/{}", id);
    Response::redirect_301(destination)
}

pub fn redirect_fanbox(client: &ureq::Agent, id: u64) -> Response {
    let url = format!("https://www.pixiv.net/fanbox/creator/{}", id);
    let response = client.get(&url).call().unwrap();
    let location = response.header("Location").unwrap().to_string();
    Response::redirect_301(location)
}
