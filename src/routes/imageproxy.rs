use crate::api::error::ApiError;

macro_rules! make_proxy {
    ($name:ident, $path:expr, $dest:tt) => {
        pub fn $name(
            client: &ureq::Agent,
            path: &str,
            request: &rouille::Request,
        ) -> Option<Result<rouille::Response, ApiError>> {
            let path = path.strip_prefix($path)?;
            let url = format!($dest, path);
            Some(proxy(&client, &url, request))
        }
    };
}

make_proxy!(imageproxy, "/imageproxy/", "https://i.pximg.net/{}");
make_proxy!(s_imageproxy, "/simg/", "https://s.pximg.net/{}");
make_proxy!(spix_imageproxy, "/spix/", "https://img-sketch.pixiv.net/{}");
make_proxy!(spxi_imageproxy, "/spxi/", "https://img-sketch.pximg.net/{}");

pub fn stamp(
    client: &ureq::Agent,
    id: u32,
    request: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let url = format!(
        "https://s.pximg.net/common/images/stamp/generated-stamps/{}_s.jpg?20180605",
        id
    );

    proxy(client, &url, request)
}

/* Note: passing these to the client should be avoided */
const FORBIDDEN_CLIENT_HEADERS: &[&str] = &["connection", "cookies", "set-cookie"];
const FORBIDDEN_SERVER_HEADERS: &[&str] =
    &["connection", "cookie", "user-agent", "host", "referer"];

fn proxy(
    client: &ureq::Agent,
    url: &str,
    request: &rouille::Request,
) -> Result<rouille::Response, ApiError> {
    let mut req = client.get(url);

    for header in request
        .headers()
        .filter(|(h, _)| !FORBIDDEN_SERVER_HEADERS.contains(&h.to_lowercase().as_str()))
    {
        req = req.set(header.0, header.1);
    }

    let res = req.call()?;
    let status = res.status();

    let headers = res
        .headers_names()
        .iter()
        .filter(|h| !FORBIDDEN_CLIENT_HEADERS.contains(&h.as_str()))
        .map(|s| (s.clone().into(), res.header(s).unwrap().to_owned().into()))
        .collect();

    let reader = match res.header("Content-Length").map(|s| s.parse::<usize>()) {
        Some(Ok(len)) => rouille::ResponseBody::from_reader_and_size(res.into_reader(), len),
        _ => rouille::ResponseBody::from_reader(res.into_reader()),
    };

    Ok(rouille::Response {
        status_code: status,
        headers,
        data: reader,
        upgrade: None,
    })
}
