//! Per-request tracing layer for HTTP traffic.
//!
//! Builds an `http_request` span carrying the HTTP method, matched route
//! template, and the request path (never the query string, which can contain
//! tokens). Emits one event per response whose level reflects the status
//! class: 5xx → ERROR, 4xx → WARN, otherwise INFO.

use std::time::Duration;

use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, Response, header},
};
use tower_http::{
    classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier},
    trace::TraceLayer,
};
use tracing::{Level, Span, field};

type MakeSpan = fn(&Request<Body>) -> Span;
type OnResponse = fn(&Response<Body>, Duration, &Span);
type OnFailure = fn(ServerErrorsFailureClass, Duration, &Span);

pub fn layer()
-> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, MakeSpan, (), OnResponse, (), (), OnFailure>
{
    TraceLayer::new_for_http()
        .make_span_with(make_span as MakeSpan)
        .on_request(())
        .on_response(on_response as OnResponse)
        .on_body_chunk(())
        .on_eos(())
        .on_failure(on_failure as OnFailure)
}

fn make_span(request: &Request<Body>) -> Span {
    let method = request.method().as_str();
    let path = request.uri().path();

    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str())
        .unwrap_or(path);

    tracing::info_span!(
        "http_request",
        method = %method,
        route = %route,
        path = %path,
        status = field::Empty,
        duration_ms = field::Empty,
        response_size = field::Empty,
    )
}

fn on_response(response: &Response<Body>, latency: Duration, span: &Span) {
    let status = response.status().as_u16();
    let duration_ms = latency.as_secs_f64() * 1000.0;
    let response_size = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    span.record("status", status);
    span.record("duration_ms", duration_ms);
    if let Some(size) = response_size {
        span.record("response_size", size);
    }

    emit_response_event(span, status);
}

/// Emits the per-response event at the level matching the status class.
// `event!` requires a const level, and its expansion inflates the metric.
#[expect(clippy::cognitive_complexity)]
fn emit_response_event(span: &Span, status: u16) {
    match status {
        500.. => tracing::event!(parent: span, Level::ERROR, "http request"),
        400.. => tracing::event!(parent: span, Level::WARN, "http request"),
        _ => tracing::event!(parent: span, Level::INFO, "http request"),
    }
}

fn on_failure(error: ServerErrorsFailureClass, latency: Duration, span: &Span) {
    let duration_ms = latency.as_secs_f64() * 1000.0;
    tracing::event!(
        parent: span,
        Level::ERROR,
        duration_ms = duration_ms,
        error = %error,
        "http request failed",
    );
}
