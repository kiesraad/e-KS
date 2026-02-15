use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate, filters,
    form::FormData,
    list_submitters::ListSubmitter,
    redirect_success,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterForm},
};

use super::SubstituteSubmitterUpdatePath;

#[derive(Template)]
#[template(path = "substitute_list_submitters/update.html")]
struct SubstituteSubmitterUpdateTemplate {
    substitute_submitter: SubstituteSubmitter,
    form: FormData<SubstituteSubmitterForm>,
}

pub async fn update_substitute_submitter(
    _: SubstituteSubmitterUpdatePath,
    context: Context,
    substitute_submitter: SubstituteSubmitter,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        SubstituteSubmitterUpdateTemplate {
            form: FormData::new_with_data(
                substitute_submitter.clone().into(),
                &context.csrf_tokens,
            ),
            substitute_submitter,
        },
        context,
    )
    .into_response())
}

pub async fn update_substitute_submitter_submit(
    _: SubstituteSubmitterUpdatePath,
    context: Context,
    substitute_submitter: SubstituteSubmitter,
    State(store): State<AppStore>,
    Form(form): Form<SubstituteSubmitterForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&substitute_submitter, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            SubstituteSubmitterUpdateTemplate {
                substitute_submitter,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(substitute_submitter) => {
            substitute_submitter.update(&store).await?;

            Ok(redirect_success(ListSubmitter::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryParamState;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    use crate::{
        AppError, AppStore, Context,
        political_groups::PoliticalGroupId,
        substitute_list_submitters::SubstituteSubmitterId,
        test_utils::{
            response_body_string, sample_political_group, sample_substitute_submitter,
            sample_substitute_submitter_form,
        },
    };

    #[tokio::test]
    async fn update_substitute_submitter_renders_existing_submitter() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let sub_submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(sub_submitter_id);

        political_group.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let response = update_substitute_submitter(
            SubstituteSubmitterUpdatePath { sub_submitter_id },
            Context::new_test_without_db(),
            substitute_submitter.clone(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(substitute_submitter.name.last_name.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn update_substitute_submitter_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let sub_submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(sub_submitter_id);

        political_group.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.name.last_name = "Updated".to_string();

        let response = update_substitute_submitter_submit(
            SubstituteSubmitterUpdatePath { sub_submitter_id },
            context,
            substitute_submitter.clone(),
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
        assert_eq!(
            location,
            ListSubmitter::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let updated = store.get_substitute_submitter(sub_submitter_id)?;
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_substitute_submitter_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let sub_submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(sub_submitter_id);

        political_group.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = update_substitute_submitter_submit(
            SubstituteSubmitterUpdatePath { sub_submitter_id },
            context,
            substitute_submitter.clone(),
            State(store),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}
