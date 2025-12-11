//! HTTP middleware

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{info, Span};

/// Request logging middleware
pub async fn log_request(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let span = Span::current();

    span.record("method", method.as_str());
    span.record("uri", uri.path());

    info!("Incoming request");

    let response = next.run(req).await;

    info!(status = response.status().as_u16(), "Request completed");

    Ok(response)
}

/// Rate limiting middleware (TODO)
pub async fn rate_limit(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // TODO: Implement rate limiting
    Ok(next.run(req).await)
}
