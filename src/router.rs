//! Builds the application Axum router and wires feature routes.
//! Used by the server startup to assemble all routes.

use axum::{
    Router,
    http::{HeaderName, HeaderValue, header},
    middleware,
    routing::get,
};
use tower_http::{csrf::CsrfLayer, set_header::SetResponseHeaderLayer};

use crate::{
    AppState, audit_log, candidate_lists, candidates, common, csb, csb_store_middleware,
    db_gate_middleware, eks_key_middleware, finalise, health_router, http_trace, list_designation,
    list_submitters, name_authorisations, persons, political_groups, render_error_pages,
    session_middleware, store_middleware, substitute_list_submitters, utils::bag,
};

pub fn create(state: AppState) -> Router<AppState> {
    let app_router = app_feature_router();

    #[cfg(feature = "dev-features")]
    let dev_router = Router::new().route(
        crate::app::dev_login::DEV_LOGIN_PATH,
        get(crate::app::dev_login::dev_login),
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
    let csb_router = csb::index::router()
        .merge(csb::audit_log::router())
        .merge(csb::examination::router())
        .merge(csb::import::router())
        .merge(csb::monitoring::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csb_store_middleware,
        ));

    // These routes need a session but NOT store middleware: select-election runs
    // before a stream_id is chosen, and /language must stay reachable for CSB
    // (committee) sessions that store_middleware redirects off app routes.
    // CSRF token verification for every mutating request, directly inside the
    // session middleware so no handler can forget it.
    let app_router = app_router
        .merge(common::session_only_router())
        .merge(csb_router)
        .layer(middleware::from_fn(crate::csrf_middleware))
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
        .merge(common::public_router());

    let router = apply_security_headers(router)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            db_gate_middleware,
        ))
        .layer(http_trace::layer());

    // The health probe is polled continuously; merging it
    // after the trace layer keeps those requests out of the request log
    let router = router.merge(health_router());

    #[cfg(feature = "livereload")]
    let router = router.merge(crate::utils::livereload::livereload_router());

    let router = mount_static_assets(router).nest("/.well-known", common::wellknown_router());

    router
        .layer(csrf_layer())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            eks_key_middleware,
        ))
}

/// Global fetch-metadata CSRF protection (`Sec-Fetch-Site`/`Origin` checks),
/// backstopping the per-session token middleware. `/saml/sp/logout` is exempt:
/// it receives cross-site IdP POSTs by design and validates SAML signatures.
fn csrf_layer() -> CsrfLayer {
    CsrfLayer::new().with_insecure_bypass(|_, uri| uri.path() == "/saml/sp/logout")
}

/// The application's feature routes (everything that sits behind the session
/// and store middleware). Kept separate from [`create`] so the wiring of
/// global layers stays readable.
fn app_feature_router() -> Router<AppState> {
    Router::new()
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
        .merge(substitute_list_submitters::router())
}

/// Static security response headers shared by every response. HSTS is added
/// separately on the TLS listener (see `core::server`), only over https.
fn apply_security_headers(mut router: Router<AppState>) -> Router<AppState> {
    // Deny all powerful browser features by default; the app uses none.
    const PERMISSIONS_POLICY: &str = "accelerometer=(), autoplay=(), camera=(), display-capture=(), encrypted-media=(), fullscreen=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), midi=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), xr-spatial-tracking=()";

    let headers: [(HeaderName, &'static str); 7] = [
        (
            header::CONTENT_SECURITY_POLICY,
            "default-src 'none'; base-uri 'none'; connect-src 'self'; form-action 'self'; script-src 'self'; style-src 'self'; font-src 'self'; img-src 'self'; frame-ancestors 'none';",
        ),
        (header::X_FRAME_OPTIONS, "DENY"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "same-origin"),
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            "same-origin",
        ),
        (
            HeaderName::from_static("cross-origin-resource-policy"),
            "same-origin",
        ),
        (
            HeaderName::from_static("permissions-policy"),
            PERMISSIONS_POLICY,
        ),
    ];

    for (name, value) in headers {
        router = router.layer(SetResponseHeaderLayer::if_not_present(
            name,
            HeaderValue::from_static(value),
        ));
    }
    router
}

/// Mount the cache-busted `/static` asset routes: served from the embedded
/// bundle in release builds, proxied to the dev asset server otherwise.
fn mount_static_assets(router: Router<AppState>) -> Router<AppState> {
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
        let mut session = crate::Session::new_test();
        session.set_stream_id(crate::StreamId::new());
        session.set_current_election(crate::ElectionConfig::EK27);
        let token = session.token_string();
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
    async fn db_gate_serves_maintenance_when_unhealthy() {
        let state = AppState::new_for_tests().await;
        // Simulate the prober (or a request handler) tripping the gate.
        state.db_health.mark_unavailable("test: database down");

        let app: Router = create(state.clone()).with_state(state.clone());

        let request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers().get("retry-after").unwrap(), "30");
        let body = response_body_string(response).await;
        assert!(body.contains("Tijdelijk niet beschikbaar"));
    }

    #[tokio::test]
    async fn db_gate_exempts_health_endpoint_when_unhealthy() {
        let state = AppState::new_for_tests().await;
        state.db_health.mark_unavailable("test: database down");

        let app: Router = create(state.clone()).with_state(state.clone());

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        // The health probe must keep answering (memory backend is reachable),
        // not be swallowed by the maintenance gate.
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// A cross-site POST is rejected by the global CSRF layer before it
    /// reaches any session or handler logic.
    #[tokio::test]
    async fn cross_site_post_is_rejected_by_csrf_layer() {
        let state = AppState::new_for_tests().await;
        let app: Router = create(state.clone()).with_state(state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/switch-election")
            .header("sec-fetch-site", "cross-site")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// A same-origin POST passes the CSRF layer: without a session cookie it
    /// reaches the session middleware, which redirects to login.
    #[tokio::test]
    async fn same_origin_post_passes_csrf_layer() {
        let state = AppState::new_for_tests().await;
        let app: Router = create(state.clone()).with_state(state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/switch-election")
            .header("sec-fetch-site", "same-origin")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    /// The SAML single-logout endpoint receives cross-site POSTs from the IdP
    /// by design and must bypass the CSRF layer (it validates SAML signatures
    /// instead).
    #[tokio::test]
    async fn cross_site_post_to_saml_logout_bypasses_csrf_layer() {
        let state = AppState::new_for_tests().await;
        let app: Router = create(state.clone()).with_state(state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/saml/sp/logout")
            .header("sec-fetch-site", "cross-site")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.expect("response");

        assert_ne!(response.status(), StatusCode::FORBIDDEN);
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
        let token = session.token_string();
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
