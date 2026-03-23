//! Builds the application Axum router and wires feature routes.
//! Used by the server startup to assemble all routes.

use axum::{
    Router,
    http::{HeaderValue, header},
    middleware,
    routing::get,
};
use tower_http::set_header::SetResponseHeaderLayer;

#[cfg(feature = "http-logging")]
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::{
    AppState, authorised_agents, candidate_lists, candidates, common, list_submitters, persons,
    political_groups, render_error_pages, session_middleware, store_middleware, submit,
    substitute_list_submitters,
};

pub fn create(state: AppState) -> Router<AppState> {
    let app_router = Router::new()
        .merge(common::router())
        .merge(persons::router())
        .merge(political_groups::router())
        .merge(authorised_agents::router())
        .merge(list_submitters::router())
        .merge(substitute_list_submitters::router())
        .merge(submit::router())
        .merge(candidate_lists::router())
        .merge(candidates::router());

    #[cfg(feature = "dev-features")]
    let bag_service_url = crate::get_env("BAG_SERVICE_URL", "http://localhost:8090")
        .expect("BAG_SERVICE_URL must be set in dev-features mode");

    #[cfg(feature = "dev-features")]
    let dev_router = Router::new()
        .route(
            crate::auth::dev_login::DEV_LOGIN_PATH,
            get(crate::auth::dev_login::dev_login),
        )
        .route(
            "/lookup",
            crate::utils::proxy::proxy_handler(&bag_service_url),
        )
        .route(
            "/suggest",
            crate::utils::proxy::proxy_handler(&bag_service_url),
        );

    let app_router = app_router
        .fallback(get(common::not_found))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            render_error_pages,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            store_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ));

    #[cfg(feature = "dev-features")]
    let router = Router::new().merge(dev_router).merge(app_router);

    #[cfg(not(feature = "dev-features"))]
    let router = app_router;

    let router = router
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; base-uri 'none'; connect-src 'self'; form-action 'self'; script-src 'self'; style-src 'self'; font-src 'self'; img-src 'self'; frame-ancestors 'none';"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("same-origin"),
        ));

    #[cfg(feature = "http-logging")]
    let router = router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
            .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
    );

    #[cfg(feature = "livereload")]
    let router = router.merge(crate::utils::livereload::livereload_router());

    #[cfg(feature = "memory-serve")]
    let router = router.nest(
        "/static",
        memory_serve::load!().index_file(None).into_router(),
    );

    #[cfg(not(feature = "memory-serve"))]
    let router = router.nest(
        "/static",
        Router::new().fallback(crate::utils::proxy::proxy_handler("http://localhost:8888")),
    );

    router
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    use crate::{AppState, test_utils::response_body_string};

    #[tokio::test]
    async fn index_route_renders_index() {
        let state = AppState::new_for_tests().await;
        let app: Router = create(state.clone()).with_state(state.clone());

        let mut request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let session = crate::Session::new();
        let token = session.token().to_exposed_string();
        state.sessions.insert(session);
        let store = crate::AppStore::new_for_test();
        request.headers_mut().insert(
            header::COOKIE,
            format!("{}={}", crate::SESSION_COOKIE_NAME, token)
                .parse()
                .unwrap(),
        );
        request.extensions_mut().insert(store);
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Kiesraad - Kandidaatstelling"));
    }

    #[tokio::test]
    async fn fallback_route_renders_not_found() {
        let state = AppState::new_for_tests().await;
        let app: Router = create(state.clone()).with_state(state.clone());

        let mut request = Request::builder()
            .uri("/missing")
            .body(Body::empty())
            .unwrap();
        let session = crate::Session::new();
        let token = session.token().to_exposed_string();
        state.sessions.insert(session);
        let store = crate::AppStore::new_for_test();
        request.headers_mut().insert(
            header::COOKIE,
            format!("{}={}", crate::SESSION_COOKIE_NAME, token)
                .parse()
                .unwrap(),
        );
        request.extensions_mut().insert(store);
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        assert!(body.contains("Pagina niet gevonden"));
    }
}
