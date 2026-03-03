use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    authorised_agents::AuthorisedAgent,
    filters,
    form::{Form, FormData},
    list_submitters::ListSubmitter,
    political_groups::{PoliticalGroup, PoliticalGroupForm, PoliticalGroupSteps},
    redirect_success,
};

use super::PoliticalGroupUpdatePath;

#[derive(Template)]
#[template(path = "political_groups/pages/update.html")]
struct PoliticalGroupUpdateTemplate {
    form: FormData<PoliticalGroupForm>,
    steps: PoliticalGroupSteps,
}

pub async fn update_political_group(
    _: PoliticalGroupUpdatePath,
    context: Context,
    store: AppStore,
    political_group: PoliticalGroup,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(&store)?;

    Ok(HtmlTemplate(
        PoliticalGroupUpdateTemplate {
            form: FormData::new_with_data(
                political_group.clone().into(),
                &context.session.csrf_tokens,
            ),
            steps,
        },
        context,
    )
    .into_response())
}

pub async fn update_political_group_submit(
    _: PoliticalGroupUpdatePath,
    context: Context,
    political_group: PoliticalGroup,
    store: AppStore,
    Form(form): Form<PoliticalGroupForm>,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(&store)?;

    match form.validate_update(&political_group, &context.session.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PoliticalGroupUpdateTemplate {
                form: form_data,
                steps,
            },
            context,
        )
        .into_response()),
        Ok(political_group) => {
            political_group.update(&store).await?;

            Ok(redirect_success(AuthorisedAgent::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        authorised_agents::AuthorisedAgentId,
        test_utils::{response_body_string, sample_authorised_agent, sample_political_group_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn update_political_group_renders_existing_data() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let political_group = store.get_political_group();

        let response = update_political_group(
            PoliticalGroupUpdatePath {},
            Context::new_test_without_db(),
            store,
            political_group,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("Kiesraad Demo"));

        Ok(())
    }

    #[tokio::test]
    async fn update_political_group_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let political_group = store.get_political_group();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = sample_political_group_form(&csrf_token);

        let response = update_political_group_submit(
            PoliticalGroupUpdatePath {},
            context,
            political_group,
            store.clone(),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(
            location,
            AuthorisedAgent::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let updated = store.get_political_group();
        assert_eq!(updated.long_list_allowed, Some(true));
        assert_eq!(
            updated.legal_name.as_deref().map(|v| v.to_string()),
            Some("Updated Legal Name".to_string())
        );
        assert_eq!(
            updated.display_name.as_deref().map(|v| v.to_string()),
            Some("Updated Display Name".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn update_political_group_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let mut form = sample_political_group_form(&csrf_token);

        form.display_name = "!".to_string(); // Invalid value

        let response = update_political_group_submit(
            PoliticalGroupUpdatePath {},
            context,
            store.get_political_group(),
            store,
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The value is too short"));

        Ok(())
    }
}
