use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use axum_extra::routing::TypedPath;

use crate::{
    log::{debug, error, trace},
    server::lib::{fetcher::fetch_image_url, filter_headers, CLIENT_HEADERS_KEEP},
};

#[derive(TypedPath, serde::Deserialize, Debug)]
#[typed_path("/img/*url")]
pub struct ImageProxy {
    url: String,
}

pub async fn redirect_to_page_image(
    ImageProxy { url }: ImageProxy,
    headers: HeaderMap,
) -> impl IntoResponse {
    let headers = filter_headers(&headers, &CLIENT_HEADERS_KEEP);

    trace!(url = ?url, headers = ?headers, "redirecting to image url");

    match fetch_image_url(&url, headers).await {
        Ok(url) => {
            debug!(url = ?url, "got image url");

            Redirect::temporary(&url).into_response()
        }
        Err(e) => {
            error!(error = %e, "got error proxying url");

            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
