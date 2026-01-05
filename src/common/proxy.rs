//! Proxy handler for forwarding requests to an upstream server.
//! See <https://github.com/tokio-rs/axum/blob/main/examples/reverse-proxy/src/main.rs>
//! Note that "legacy" is currently the only stable high-level client in hyper-util (and is not deprecated).

use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};
use hyper_util::rt::TokioExecutor;
use std::{
    cell::LazyCell,
    sync::{Arc, Mutex},
};

use crate::AppState;

pub fn proxy_handler(upstream: impl Into<String>) -> axum::routing::MethodRouter<AppState> {
    let upstream = upstream.into();
    let client = Arc::new(Mutex::new(LazyCell::new(|| {
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new()).build_http()
    })));

    axum::routing::any(move |mut req: Request<Body>| {
        let upstream = upstream.clone();
        let client = Arc::clone(&client);

        async move {
            let path = req.uri().path();
            let path_query = req
                .uri()
                .path_and_query()
                .map(|v| v.as_str())
                .unwrap_or(path);

            let uri = format!("{upstream}{path_query}");

            *req.uri_mut() = Uri::try_from(uri).map_err(|_| StatusCode::BAD_REQUEST)?;

            let client = {
                let client = client.lock().map_err(|_| StatusCode::BAD_REQUEST)?;
                LazyCell::force(&client).clone()
            };

            Ok::<Response, StatusCode>(
                client
                    .request(req)
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?
                    .into_response(),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use sqlx::PgPool;
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    use crate::test_utils::response_body_string;

    #[sqlx::test]
    async fn proxy_forwards_requests_to_upstream(pool: PgPool) -> Result<(), sqlx::Error> {
        let upstream_router = Router::new().route("/up", get(|| async { "ok" }));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let upstream = tokio::spawn(async move {
            axum::serve(listener, upstream_router).await.unwrap();
        });

        let app_state = AppState::new_for_tests(pool);
        let app = Router::new()
            .route("/up", proxy_handler(format!("http://{addr}")))
            .with_state(app_state);

        let response = app
            .oneshot(Request::builder().uri("/up").body(Body::empty()).unwrap())
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert_eq!(body, "ok");

        upstream.abort();

        Ok(())
    }
}
