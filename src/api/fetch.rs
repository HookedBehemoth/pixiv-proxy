use crate::api::{common::ApiResponse, error::ApiError};
use serde::Deserialize;
use ureq::Body;

fn fetch_json_internal<T>(response: ureq::http::Response<Body>) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    let status = response.status();
    let mut body = response.into_body();
    if !status.is_success() {
        Err(ApiError::External(
            status.as_u16(),
            body.read_to_string().map_or("".into(), |s| s.into()),
        ))
    } else {
        match body.read_json::<T>() {
            Ok(res) => Ok(res),
            Err(err) => Err(ApiError::Internal(err.to_string().into())),
        }
    }
}
pub(crate) fn fetch_json<T>(client: &ureq::Agent, url: &str) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    fetch_json_internal(client.get(url).call()?)
}

pub(crate) fn post_and_fetch_json<T>(
    client: &ureq::Agent,
    url: &str,
    body: String,
) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    fetch_json_internal(
        client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send(&body)?,
    )
}

/* Fetch from pixiv ajax API */
pub(crate) fn fetch<T>(client: &ureq::Agent, url: &str) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    let response = fetch_json::<ApiResponse<T>>(client, url)?;

    if response.error || response.body.is_none() {
        Err(ApiError::External(
            400,
            response.message.map_or("".into(), |s| s.into()),
        ))
    } else {
        Ok(response.body.unwrap())
    }
}
