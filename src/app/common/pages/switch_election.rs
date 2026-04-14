use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AnyLocale, AppError, AppState, Context, CsrfToken, ElectionConfig, HtmlTemplate, Province,
    Session, TokenValue, WaterCouncil, common::SwitchElectionForm, filters,
};

use super::SwitchElectionPath;

#[derive(Template)]
#[template(path = "common/pages/switch_election.html")]
struct SwitchElectionTemplate {
    elections: Vec<ElectionConfig>,
    existing_elections: Vec<ElectionConfig>,
    current_election: ElectionConfig,
    title_locale: AnyLocale,
    current_type: &'static str,
    selected_region: Option<&'static str>,
    provinces: &'static [Province],
    water_councils: &'static [WaterCouncil],
    csrf_token: CsrfToken,
}

pub async fn switch_election(
    _: SwitchElectionPath,
    State(state): State<AppState>,
    context: Context,
) -> Result<Response, AppError> {
    let existing_elections = match &context.session.bsn {
        Some(bsn) => state.existing_elections_for_bsn(bsn).await?,
        None => Vec::new(),
    };

    let csrf_token = context.session.csrf_tokens.issue();
    let elections = ElectionConfig::type_options();

    Ok(HtmlTemplate(
        SwitchElectionTemplate {
            current_election: context.election,
            title_locale: AnyLocale::from(context.session.locale),
            current_type: context.election.code(),
            selected_region: context.election.region_code(),
            provinces: Province::ALL,
            water_councils: WaterCouncil::ALL,
            elections,
            existing_elections,
            csrf_token,
        },
        context,
    )
    .into_response())
}

pub async fn switch_election_submit(
    _: SwitchElectionPath,
    State(state): State<AppState>,
    mut session: Session,
    axum::Form(form): axum::Form<SwitchElectionForm>,
) -> Result<Response, AppError> {
    if !session
        .csrf_tokens
        .consume(&TokenValue(form.csrf_token.clone()))
    {
        return Err(AppError::CsrfTokenInvalid);
    }

    let Some(election) = form.into_election_config() else {
        return Ok(Redirect::to("/switch-election").into_response());
    };

    let stream_id = match &session.bsn {
        Some(bsn) => state.bsn_id_deriver.derive_stream_id(bsn, election),
        None => crate::StreamId::new(),
    };

    // Short-circuit if already on this election (same BSN + election = same stream)
    if Some(stream_id) == session.stream_id {
        return Ok(Redirect::to("/").into_response());
    }

    // Ensure the stream/store exists for the new election
    state.store_for_stream(stream_id, election).await?;

    session.set_stream_id(stream_id);
    state.sessions.insert(session);

    Ok(Redirect::to("/").into_response())
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
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        // First request to get a session cookie
        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/switch-election")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(first.status(), StatusCode::OK);

        let cookie = first
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .expect("cookie");

        // Issue a CSRF token for the session
        let token_value = cookie.split_once('=').map(|(_, v)| v).expect("token value");
        let session = state.sessions.get(token_value).expect("session");
        let csrf_token = session.csrf_tokens.issue();

        // Submit switch to PS27 Groningen
        let body = format!(
            "csrf_token={}&election=PS27&region_province=GR",
            csrf_token.value
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/switch-election")
                    .header(header::COOKIE, cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");

        // Verify session stream was updated
        let session = state.sessions.get(token_value).expect("session");
        let expected_stream_id = state.bsn_id_deriver.derive_stream_id(
            &session.bsn.clone().expect("bsn"),
            ElectionConfig::PS27(Province::GR),
        );
        assert_eq!(session.stream_id, Some(expected_stream_id));
    }
}
