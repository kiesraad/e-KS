use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AnyLocale, AppError, AppState, Context, ElectionConfig, HtmlTemplate, Province, Session,
    WaterCouncil, common::SwitchElectionForm, filters,
};

use super::{IndexPath, SwitchElectionPath};

#[derive(Template)]
#[template(path = "app/common/pages/switch_election.html")]
struct SwitchElectionTemplate {
    elections: Vec<ElectionConfig>,
    existing_elections: Vec<ElectionConfig>,
    current_election: ElectionConfig,
    title_locale: AnyLocale,
    current_type: &'static str,
    selected_region: Option<&'static str>,
    provinces: &'static [Province],
    water_councils: &'static [WaterCouncil],
}

pub async fn switch_election(
    _: SwitchElectionPath,
    State(state): State<AppState>,
    context: Context,
) -> Result<Response, AppError> {
    let existing_elections = existing_elections_for(&state, &context.session).await?;

    Ok(HtmlTemplate(
        SwitchElectionTemplate {
            current_election: context.election,
            title_locale: AnyLocale::from(context.session.locale),
            current_type: context.election.code(),
            selected_region: context.election.region_code(),
            provinces: Province::ALL,
            water_councils: WaterCouncil::ALL,
            elections: ElectionConfig::type_options(),
            existing_elections,
        },
        context,
    )
    .into_response())
}

async fn existing_elections_for(
    state: &AppState,
    session: &Session,
) -> Result<Vec<ElectionConfig>, AppError> {
    match session.stream_id {
        Some(stream_id) => state.existing_elections_for_stream(stream_id).await,
        None => Ok(Vec::new()),
    }
}

pub async fn switch_election_submit(
    _: SwitchElectionPath,
    State(state): State<AppState>,
    mut session: Session,
    axum::Form(form): axum::Form<SwitchElectionForm>,
) -> Result<Response, AppError> {
    let Some(election) = form.into_election_config() else {
        return Ok(Redirect::to(&SwitchElectionPath.to_string()).into_response());
    };

    // Short-circuit if already on this election.
    if session.current_election == Some(election) {
        return Ok(Redirect::to(&IndexPath.to_string()).into_response());
    }

    let Some(stream_id) = session.stream_id else {
        return Ok(Redirect::to(&SwitchElectionPath.to_string()).into_response());
    };

    // Ensure the store exists for the new election.
    state.store_for_stream(stream_id, election, false).await?;

    session.set_current_election(election);
    state.sessions.insert(session).await;

    Ok(Redirect::to(&IndexPath.to_string()).into_response())
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

    use crate::{AppState, ElectionConfig, Province, session_middleware, store_middleware};

    use super::*;

    #[tokio::test]
    async fn switch_election_submit_changes_session_election() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_get(switch_election)
            .typed_post(switch_election_submit)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                store_middleware,
            ))
            .layer(middleware::from_fn(crate::csrf_middleware))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        // Pre-create a session with a stream and a starting election.
        let stream_id = crate::StreamId::new();
        state
            .store_for_stream(stream_id, ElectionConfig::EK27, false)
            .await
            .expect("store");

        let mut session = crate::Session::new();
        session.set_stream_id(stream_id);
        session.set_current_election(ElectionConfig::EK27);
        let token_value = session.token_string();
        let csrf_token = session.csrf_token();
        state.sessions.insert(session).await;

        let cookie = format!("{}={}", crate::SESSION_COOKIE_NAME, token_value);

        // Submit switch to PS27 Groningen
        let body = format!("csrf_token={csrf_token}&election=PS27&region_province=GR");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/switch-election")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");

        // Verify session current_election was updated (stream_id stays the same).
        let session = state
            .sessions
            .get(&token_value)
            .await
            .expect("load session")
            .expect("session");
        assert_eq!(session.stream_id, Some(stream_id));
        assert_eq!(
            session.current_election,
            Some(ElectionConfig::PS27(Province::GR))
        );
    }
}
