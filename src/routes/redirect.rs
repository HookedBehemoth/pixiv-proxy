use crate::api::error::ApiError;

pub fn jump(path: &rouille::Request) -> Result<rouille::Response, ApiError> {
    let destination = path.raw_query_string();
    let destination = percent_encoding::percent_decode_str(destination)
        .decode_utf8_lossy()
        .into_owned();

    Ok(rouille::Response::redirect_301(destination))
}

pub fn legacy_illust(query: &rouille::Request) -> Result<rouille::Response, ApiError> {
    let illust_id = query
        .get_param("illust_id")
        .ok_or_else(|| ApiError::External(402, "Missing illust ID".into()))?;
    let destination = format!("/artworks/{}", illust_id);
    Ok(rouille::Response::redirect_301(destination))
}

pub fn fanbox(client: &ureq::Agent, id: u64) -> Result<rouille::Response, ApiError> {
    let url = format!("https://www.pixiv.net/fanbox/creator/{}", id);
    let response = client.get(&url).call()?;

    let destination = response
        .header("location")
        .ok_or_else(|| ApiError::External(404, "User not found".into()))?;

    Ok(rouille::Response::redirect_301(destination.to_owned()))
}
