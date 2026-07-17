use axum::response::Redirect;
use axum_extra::{TypedHeader, headers};

use crate::{AppEvent, AppStore, common::HideDownloadWarningPath};

pub async fn hide_download_warning(
    _: HideDownloadWarningPath,
    TypedHeader(referer): TypedHeader<headers::Referer>,
    store: AppStore,
) -> Result<Redirect, crate::AppError> {
    store.update(AppEvent::HideDownloadWarning).await?;
    Ok(Redirect::to(&referer.to_string()))
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        middleware,
    };
    use axum_extra::routing::RouterExt;
    use tower::ServiceExt;

    use crate::{AppEvent, AppState, ElectionConfig, session_middleware, store_middleware};

    use super::*;

    #[tokio::test]
    async fn hide_download_warning_records_event_and_redirects() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_post(hide_download_warning)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                store_middleware,
            ))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        let stream_id = crate::StreamId::new();
        let store = crate::AppStore::own(
            state
                .store_for_stream(stream_id, ElectionConfig::EK27, false)
                .await
                .expect("store"),
        );

        let mut session = crate::Session::new();
        session.set_stream_id(stream_id);
        session.set_current_election(ElectionConfig::EK27);
        let token = session.token_string();
        let csrf = session.csrf_token().clone();
        state.sessions.insert(session).await;

        // after download, the warning should show
        store
            .update(AppEvent::DownloadFile {
                file_name: "documents.zip".to_string(),
                download_path: "/download".to_string(),
            })
            .await
            .unwrap();
        assert!(store.should_show_download_warning());

        let request = Request::builder()
            .method("POST")
            .uri("/hide-download-warning")
            .header(header::REFERER, "https://example.com/candidate-lists")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(
                header::COOKIE,
                format!("{}={}", crate::SESSION_COOKIE_NAME, token),
            )
            .body(Body::from(format!("csrf_token={csrf}")))
            .unwrap();

        let response = app.oneshot(request).await.expect("response");

        // we should be redirected and the warning should no longer show
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://example.com/candidate-lists",
        );
        assert!(!store.should_show_download_warning());
    }
}
