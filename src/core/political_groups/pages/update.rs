use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    authorised_agents::AuthorisedAgent,
    filters,
    form::FormData,
    list_submitters::ListSubmitter,
    political_groups::{PoliticalGroup, PoliticalGroupForm, PoliticalGroupSteps},
};

use super::PoliticalGroupUpdatePath;

#[derive(Template)]
#[template(path = "political_groups/update.html")]
struct PoliticalGroupUpdateTemplate {
    form: FormData<PoliticalGroupForm>,
    steps: PoliticalGroupSteps,
}

pub async fn update_political_group(
    _: PoliticalGroupUpdatePath,
    context: Context,
    State(store): State<AppStore>,
    political_group: PoliticalGroup,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(store.clone())?;

    Ok(HtmlTemplate(
        PoliticalGroupUpdateTemplate {
            form: FormData::new_with_data(political_group.clone().into(), &context.csrf_tokens),
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
    State(store): State<AppStore>,
    Form(form): Form<PoliticalGroupForm>,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(store.clone())?;

    match form.validate_update(&political_group, &context.csrf_tokens) {
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

            Ok(Redirect::to(&PoliticalGroup::update_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context,
        authorised_agents::AuthorisedAgentId,
        political_groups::PoliticalGroupId,
        test_utils::{
            response_body_string, sample_authorised_agent, sample_political_group,
            sample_political_group_form,
        },
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;

    #[tokio::test]
    async fn update_political_group_renders_existing_data() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);

        political_group.create(&store).await?;

        let response = update_political_group(
            PoliticalGroupUpdatePath {},
            Context::new_test_without_db(),
            State(store),
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
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_political_group_form(&csrf_token);

        let response = update_political_group_submit(
            PoliticalGroupUpdatePath {},
            context,
            political_group,
            State(store.clone()),
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
        assert_eq!(location, PoliticalGroup::update_path());

        let updated = store.get_political_group()?;
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
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_political_group_form(&csrf_token);

        form.display_name = "!".to_string(); // Invalid value

        let response = update_political_group_submit(
            PoliticalGroupUpdatePath {},
            context,
            political_group,
            State(store),
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
