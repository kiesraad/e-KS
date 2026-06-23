//! Builds the application Axum router and wires feature routes.
//! Used by the server startup to assemble all routes.

use axum::{
    Router,
    http::{HeaderValue, header},
    middleware,
    routing::get,
};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::{
    AppState, audit_log, candidate_lists, candidates, common, csb, csb_store_middleware,
    eks_key_middleware, finalise, health_router, http_trace, list_designation, list_submitters,
    name_authorisations, persons, political_groups, render_error_pages, session_middleware,
    store_middleware, substitute_list_submitters, utils::bag,
};

pub fn create(state: AppState) -> Router<AppState> {
    let app_router = Router::new()
        .merge(audit_log::router())
        .merge(candidates::router())
        .merge(candidate_lists::router())
        .merge(common::router())
        .merge(list_designation::router())
        .merge(list_submitters::router())
        .merge(name_authorisations::router())
        .merge(persons::router())
        .merge(political_groups::router())
        .merge(finalise::router())
        .merge(substitute_list_submitters::router());

    #[cfg(feature = "dev-features")]
    let dev_router = Router::new().route(
        crate::auth::dev_login::DEV_LOGIN_PATH,
        get(crate::auth::dev_login::dev_login),
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
        ));

    // CSB routes need the session plus their own (CSB) store middleware, which
    // also gates them to committee-scoped sessions. They must NOT get the app
    // `store_middleware`, so they are merged here rather than above.
    let csb_router = csb::examination::router()
        .merge(csb::import::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csb_store_middleware,
        ));

    // These routes need a session but NOT store middleware: select-election runs
    // before a stream_id is chosen, and /language must stay reachable for CSB
    // (committee) sessions that store_middleware redirects off app routes.
    let app_router = app_router
        .merge(common::session_only_router())
        .merge(csb_router)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ));

    #[cfg(feature = "dev-features")]
    let router = Router::new().merge(dev_router).merge(app_router);

    #[cfg(not(feature = "dev-features"))]
    let router = app_router;

    let router = router
        .merge(bag::router())
        .merge(auth_service::router())
        .merge(health_router())
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
        ))
        .layer(http_trace::layer());

    #[cfg(feature = "livereload")]
    let router = router.merge(crate::utils::livereload::livereload_router());

    let code = crate::filters::cache_buster();
    let index_js = format!("/{code}-index.js");
    let index_css = format!("/{code}-index.css");

    #[cfg(feature = "memory-serve")]
    let router = {
        let memory_serve = memory_serve::load!()
            .index_file(None)
            .add_alias(index_js.leak(), "/index.js")
            .add_alias(index_css.leak(), "/index.css");

        router.nest("/static", memory_serve.into_router())
    };

    #[cfg(not(feature = "memory-serve"))]
    let router = router.nest(
        "/static",
        Router::new().fallback(crate::proxy_handler(
            "http://localhost:8888",
            vec![
                (index_js, "/index.js".to_string()),
                (index_css, "/index.css".to_string()),
            ],
        )),
    );

    let router = router.nest("/.well-known", common::wellknown_router());

    router.layer(middleware::from_fn_with_state(
        state.clone(),
        eks_key_middleware,
    ))
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
        let mut session = crate::Session::new_test();
        session.set_stream_id(crate::StreamId::new());
        session.set_current_election(crate::ElectionConfig::EK27);
        let token = session.token().to_exposed_string();
        state.sessions.insert(session).await;
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
        let mut session = crate::Session::new_test();
        session.set_stream_id(crate::StreamId::new());
        session.set_current_election(crate::ElectionConfig::EK27);
        let token = session.token().to_exposed_string();
        state.sessions.insert(session).await;
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
