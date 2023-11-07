use axum::{
    body::StreamBody,
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
        Ok((status, headers, resp)) => {
            (status, headers, StreamBody::new(resp.bytes_stream())).into_response()
        }
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
) -> anyhow::Result<(StatusCode, HeaderMap, reqwest::Response)> {
    let resp = reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()?
        .get(url)
        .send()
        .await?
        .error_for_status()?;
    trace!("got image response");

    let status = resp.status();

    let headers = filter_headers(resp.headers(), &RESPONSE_HEADERS_KEEP);
    trace!(headers = ?headers, "got image headers");

    Ok((status, headers, resp))
}
