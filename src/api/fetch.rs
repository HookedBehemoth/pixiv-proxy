use crate::api::{common::ApiResponse, error::ApiError};
use serde::Deserialize;

async fn fetch_json_internal<T>(request: awc::SendClientRequest) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    match request.await {
        Ok(mut response) => {
            if !response.status().is_success() {
                Err(ApiError::External(
                    response.status().as_u16(),
                    String::from_utf8(response.body().await.unwrap().to_vec()).unwrap(),
                ))
            } else {
                match response.json::<T>().await {
                    Ok(response) => Ok(response),
                    Err(err) => Err(ApiError::Internal(err.to_string())),
                }
            }
        }
        Err(err) => Err(ApiError::Internal(err.to_string())),
    }
}
pub(crate) async fn fetch_json<T>(client: &awc::Client, url: &str) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    fetch_json_internal(client.get(url).send()).await
}

pub(crate) async fn post_and_fetch_json<T>(
    client: &awc::Client,
    url: &str,
    body: String,
) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    fetch_json_internal(client.post(url).send_body(body)).await
}

/* Fetch from pixiv ajax API */
pub(crate) async fn fetch<T>(client: &awc::Client, url: &str) -> Result<T, ApiError>
where
    T: for<'a> Deserialize<'a>,
{
    let response = fetch_json::<ApiResponse<T>>(client, url).await?;

    if response.error || response.body.is_none() {
        Err(ApiError::External(
            400,
            response.message.unwrap_or_default(),
        ))
    } else {
        Ok(response.body.unwrap())
    }
}
