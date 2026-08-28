use askama::Template;
use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    AnyLocale, AppError, AppRequestState, Context, Province, Scope, Session, SessionPageValues,
    WaterCouncil,
    common::{PgIndexPath, SelectElectionForm},
    csb::index::CsbIndexPath,
    filters,
};

use super::SelectElectionPath;

#[derive(Template)]
#[template(path = "pg/common/pages/select_election.html")]
struct SelectElectionTemplate {
    elections: Vec<crate::ElectionConfig>,
    title_locale: AnyLocale,
    provinces: &'static [Province],
    water_councils: &'static [WaterCouncil],
}

pub async fn select_election<S: AppRequestState>(
    _: SelectElectionPath,
    session: Session,
    State(state): State<S>,
) -> Result<Response, AppError> {
    if session.current_election.is_some() {
        return Ok(Redirect::to(&PgIndexPath.to_string()).into_response());
    }

    state.sessions().update(&session).await;

    let template = SelectElectionTemplate {
        elections: crate::ElectionConfig::type_options(),
        title_locale: AnyLocale::from(session.locale),
        provinces: Province::ALL,
        water_councils: WaterCouncil::ALL,
    };

    let values = SessionPageValues {
        locale: session.locale,
        csrf_token: session.csrf_token().0.clone(),
    };
    let html = template
        .render_with_values(&values)
        .map_err(AppError::TemplateError)?;

    let mut response = Html(html).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

pub async fn select_election_submit<S: AppRequestState>(
    _: SelectElectionPath,
    State(state): State<S>,
    mut session: Session,
    axum::Form(form): axum::Form<SelectElectionForm>,
) -> Result<Response, AppError> {
    // Committee sessions use CSB stores, not app stores; never create an
    // `PgStore` in their `(stream_id, election)` partition.
    if session.scope == Scope::CentralElectoralCommittee {
        return Ok(Redirect::to(&CsbIndexPath {}.to_string()).into_response());
    }

    let Some(election) = form.election_config() else {
        return Ok(Redirect::to(&SelectElectionPath.to_string()).into_response());
    };

    // Only available with the `fixtures` feature: this is a test/dev shortcut
    // into the committee (CSB) scope
    #[cfg(feature = "fixtures")]
    if form.login_as_csb() {
        let stream_id = crate::StreamId::new();
        let user = crate::CsbUser::Developer { stream_id };
        session.stream_id = Some(stream_id);
        session.scope = Scope::CentralElectoralCommittee;
        session.set_csb_user(user.clone());
        session.set_current_election(election);

        if form.load_fixtures() {
            crate::csb::import::fixture::import_csb_fixture(&state, election, user).await?;
        }

        session.rotate_csrf_token();
        state.sessions().update(&session).await;

        return Ok(Redirect::to(&CsbIndexPath {}.to_string()).into_response());
    }

    // use the stream ID derived from the authenticated login
    let Some(stream_id) = session.stream_id else {
        return Ok(Redirect::to(&SelectElectionPath.to_string()).into_response());
    };

    let _store = state
        .store_for_stream(stream_id, election, form.load_fixtures())
        .await?;

    session.set_current_election(election);
    // Invalidate forms rendered before an election was picked, so a stale tab
    // cannot submit against the election chosen here.
    session.rotate_csrf_token();
    state.sessions().update(&session).await;

    Ok(Redirect::to(&PgIndexPath.to_string()).into_response())
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

    use crate::{AppState, ElectionConfig, Session, session_middleware};

    use super::*;

    #[tokio::test]
    async fn select_election_redirects_when_election_already_set() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_get(select_election::<crate::AppState>)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        let mut session = Session::new_test();
        session.set_stream_id(crate::StreamId::new());
        session.set_current_election(ElectionConfig::EK27);
        let token = session.token_string();
        state.sessions().insert(session).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/select-election")
                    .header(
                        header::COOKIE,
                        format!("{}={}", crate::SESSION_COOKIE_NAME, token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");
    }

    #[tokio::test]
    async fn select_election_submit_sets_current_election() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_get(select_election::<crate::AppState>)
            .typed_post(select_election_submit::<crate::AppState>)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        let mut session = Session::new_test();
        session.set_stream_id(crate::StreamId::new());
        let token = session.token_string();
        state.sessions().insert(session).await;

        // Fetch page to receive a CSRF token
        let get = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/select-election")
                    .header(
                        header::COOKIE,
                        format!("{}={}", crate::SESSION_COOKIE_NAME, token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");
        assert_eq!(get.status(), StatusCode::OK);

        let session = state
            .sessions
            .get(&token)
            .await
            .expect("load session")
            .expect("session");
        let csrf = session.csrf_token();

        let body = format!("csrf_token={csrf}&election=EK27");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/select-election")
                    .header(
                        header::COOKIE,
                        format!("{}={}", crate::SESSION_COOKIE_NAME, token),
                    )
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");

        let session = state
            .sessions
            .get(&token)
            .await
            .expect("load session")
            .expect("session");
        assert_eq!(session.current_election, Some(ElectionConfig::EK27));
    }
}
