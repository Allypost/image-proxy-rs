use std::time::Duration;

use axum::{
    extract::MatchedPath,
    http::{Request, Response},
    routing::get,
    Router,
};
use axum_extra::routing::RouterExt;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    classify::ServerErrorsFailureClass,
    cors::{self, CorsLayer},
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::Span;

use super::log;
use crate::config::CONFIG;

mod lib;
mod routes;

#[derive(Default, Clone)]
struct MyMakeRequestId;

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = ulid::Ulid::new().to_string().parse().ok()?;
        Some(RequestId::new(id))
    }
}

pub async fn run() -> anyhow::Result<()> {
    log::info!("Starting image-proxy server...");

    let app = Router::new()
        .route("/", get(|| async { "image proxy" }))
        .typed_get(routes::img::redirect_to_page_image)
        .typed_get(routes::proxy::proxy_page_image)
        .layer(
            ServiceBuilder::new()
                .layer(CatchPanicLayer::new())
                .layer(SetRequestIdLayer::x_request_id(MyMakeRequestId))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &Request<_>| {
                            let matched_path = request
                                .extensions()
                                .get::<MatchedPath>()
                                .map(MatchedPath::as_str);

                            let request_id = request
                                .extensions()
                                .get::<RequestId>()
                                .and_then(|x| x.header_value().to_str().ok())
                                .unwrap_or_default();

                            let span = tracing::info_span!(
                                "request",
                                method = ?request.method(),
                                uri = %request.uri(),
                                version = ?request.version(),
                                path = matched_path.unwrap_or("/"),
                                id = %request_id,
                                headers = ?request.headers(),
                            );
                            span
                        })
                        // .on_request(move |request: &Request<_>, _span: &Span| {})
                        .on_failure(
                            move |error: ServerErrorsFailureClass,
                                  latency: Duration,
                                  _span: &Span| {
                                tracing::error!(error = ?error, latency = ?latency);
                            },
                        )
                        .on_response(
                            move |response: &Response<_>, latency: Duration, _span: &Span| {
                                tracing::info!(
                                    latency = ?latency,
                                    status = ?response.status(),
                                );
                            },
                        ),
                )
                .layer(
                    CorsLayer::new()
                        .allow_methods(cors::Any)
                        .allow_headers(cors::Any)
                        .allow_origin(cors::Any),
                )
                .layer(PropagateRequestIdLayer::x_request_id()),
        );

    let listener = std::net::TcpListener::bind((CONFIG.run.host.clone(), CONFIG.run.port))?;
    log::debug!("Started listener on {}", listener.local_addr()?);

    let server = axum::Server::from_tcp(listener)?.serve(app.into_make_service());
    log::info!("Server started on http://{}", server.local_addr());

    server.await?;
    log::info!("Shutting down...");

    Ok(())
}
