use crate::api::{common::ApiResponse, error::ApiError};
use serde::Deserialize;

pub(crate) fn fetch_generic(client: &ureq::Agent, url: &str) -> Result<ureq::Response, ApiError> {
    fn is_success(status: u16) -> bool {
        (200..300).contains(&status)
    }

    match client.get(url).call() {
        Ok(response) => {
            if !is_success(response.status()) {
                Err(ApiError::External(
                    response.status(),
                    response.into_string().unwrap_or_default(),
                ))
            } else {
                Ok(response)
            }
        }
        Err(err) => Err(ApiError::Internal(err.to_string())),
    }
}

/* Fetch from pixiv ajax API */
pub(crate) fn fetch<T>(client: &ureq::Agent, url: &str) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    let response = fetch_generic(client, url)?;

    match response.into_json::<ApiResponse<T>>() {
        Ok(response) => {
            if response.error || response.body.is_none() {
                Err(ApiError::External(
                    400,
                    response.message.unwrap_or_default(),
                ))
            } else {
                Ok(response.body.unwrap())
            }
        }
        Err(err) => Err(ApiError::Internal(err.to_string())),
    }
}
