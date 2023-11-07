use std::time::Duration;

use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::routing::TypedPath;

use crate::{
    log::{error, trace},
    server::lib::{
        fetcher::fetch_image_url, filter_headers, CLIENT_HEADERS_KEEP, RESPONSE_HEADERS_KEEP,
    },
};

#[derive(TypedPath, serde::Deserialize, Debug)]
#[typed_path("/proxy/*url")]
pub struct ImageProxy {
    url: String,
}

pub async fn proxy_page_image(
    ImageProxy { url }: ImageProxy,
    headers: HeaderMap,
) -> impl IntoResponse {
    let headers = filter_headers(&headers, &CLIENT_HEADERS_KEEP);

    trace!(url = ?url, headers = ?headers, "proxying image from url");

    let image_url = match fetch_image_url(&url, headers.clone()).await {
        Ok(url) => url,
        Err(e) => {
            error!(error = ?e, "failed to fetch image url");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to fetch image url: {}", e),
            )
                .into_response();
        }
    };

    trace!(url = ?image_url, "got image url");

    let image_bytes = get_image_bytes(&image_url, headers).await;

    match image_bytes {
        Ok(resp) => resp.into_response(),
        Err(e) => {
            error!(error = ?e, "failed to proxy image");

            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

#[tracing::instrument(skip(headers))]
async fn get_image_bytes(
    url: &str,
    headers: HeaderMap,
) -> anyhow::Result<(StatusCode, HeaderMap, Bytes)> {
    let resp = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()?
        .get(url)
        .send()
        .await?;

    trace!("got image response");

    let status = resp.status();

    let headers = resp
        .headers()
        .into_iter()
        .filter(|(name, _value)| RESPONSE_HEADERS_KEEP.contains(&name.to_string()))
        .fold(HeaderMap::new(), |mut acc, (name, value)| {
            acc.insert(name.clone(), value.clone());
            acc
        });

    trace!(headers = ?headers, "got image headers");

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("failed to get image bytes: {}", e))?;

    trace!("got image bytes");

    Ok((status, headers, bytes))
}
