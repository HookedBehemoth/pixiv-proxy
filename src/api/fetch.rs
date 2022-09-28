use crate::api::{common::ApiResponse, error::ApiError};
use serde::Deserialize;

fn fetch_json_internal<T>(response: ureq::Response) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    let status = response.status();
    if !(200..300).contains(&status) {
        Err(ApiError::External(
            response.status(),
            response.into_string().map_or("".into(), |s| s.into()),
        ))
    } else {
        match response.into_json::<T>() {
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
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&body)?,
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
