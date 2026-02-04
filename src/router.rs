//! Builds the application Axum router and wires feature routes.
//! Used by the server startup to assemble all routes.

use axum::{Router, middleware, routing::get};

#[cfg(feature = "http-logging")]
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::{
    AppState, candidate_lists, locale, pages, persons, political_groups, render_error_pages, submit,
};

pub fn create(state: AppState) -> Router<AppState> {
    let router = Router::new()
        .route("/", get(pages::index))
        .merge(persons::router())
        .merge(political_groups::router())
        .merge(submit::router())
        .merge(candidate_lists::router())
        .merge(candidate_lists::candidate_router())
        .merge(locale::locale_router());

    #[cfg(feature = "dev-features")]
    let bag_service_url =
        crate::common::config::get_env("BAG_SERVICE_URL", "http://localhost:8080")
            .expect("BAG_SERVICE_URL must be set in dev-features mode");

    #[cfg(feature = "dev-features")]
    let router = router
        .route(
            "/lookup",
            crate::common::proxy::proxy_handler(&bag_service_url),
        )
        .route(
            "/suggest",
            crate::common::proxy::proxy_handler(&bag_service_url),
        );

    #[cfg(feature = "http-logging")]
    let router = router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
            .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
    );

    #[cfg(feature = "livereload")]
    let router = router.merge(crate::common::livereload::livereload_router());

    #[cfg(feature = "memory-serve")]
    let router = router.nest(
        "/static",
        memory_serve::load!().index_file(None).into_router(),
    );

    #[cfg(not(feature = "memory-serve"))]
    let router = router.nest(
        "/static",
        Router::new().fallback(crate::common::proxy::proxy_handler("http://localhost:8888")),
    );

    router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            render_error_pages,
        ))
        .fallback(get(pages::not_found))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{AppState, test_utils::response_body_string};

    #[sqlx::test]
    async fn index_route_renders_index(pool: PgPool) {
        let state = AppState::new_for_tests(&pool).await;
        let app: Router = create(state.clone()).with_state(state);

        let request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Kiesraad - Kandidaatstelling"));
    }

    #[sqlx::test]
    async fn fallback_route_renders_not_found(pool: PgPool) {
        let state = AppState::new_for_tests(&pool).await;
        let app: Router = create(state.clone()).with_state(state);

        let request = Request::builder()
            .uri("/missing")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        assert!(body.contains("Pagina niet gevonden"));
    }
}
