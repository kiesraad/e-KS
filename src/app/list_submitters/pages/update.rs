use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, Context, Form, HtmlTemplate, Store, filters,
    form::FormData,
    list_submitters::{ListSubmitter, ListSubmitterForm},
    redirect_success,
};

use super::ListSubmitterUpdatePath;

#[derive(Template)]
#[template(path = "list_submitters/pages/update.html")]
struct ListSubmitterUpdateTemplate {
    list_submitter: ListSubmitter,
    form: FormData<ListSubmitterForm>,
}

pub async fn update_list_submitter(
    _: ListSubmitterUpdatePath,
    context: Context,
    list_submitter: ListSubmitter,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            form: FormData::new_with_data(list_submitter.clone().into(), &context.csrf_tokens),
            list_submitter,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter_submit(
    _: ListSubmitterUpdatePath,
    context: Context,
    list_submitter: ListSubmitter,
    State(store): State<Store>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&list_submitter, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterUpdateTemplate {
                list_submitter,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(list_submitter) => {
            list_submitter.update(&store).await?;

            Ok(redirect_success(ListSubmitter::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, Context, Form, QueryParamState, Store,
        list_submitters::ListSubmitterId,
        political_groups::PoliticalGroupId,
        test_utils::{
            response_body_string, sample_list_submitter, sample_list_submitter_form,
            sample_political_group,
        },
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn update_list_submitter_renders_existing_submitter() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let response = update_list_submitter(
            ListSubmitterUpdatePath { submitter_id },
            Context::new_test_without_db(),
            list_submitter.clone(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(list_submitter.name.last_name.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitter_persists_and_redirects() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.name.last_name = "Updated".to_string();

        let response = update_list_submitter_submit(
            ListSubmitterUpdatePath { submitter_id },
            context,
            list_submitter.clone(),
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

        let updated = store.get_list_submitter(submitter_id)?;
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitter_invalid_form_renders_template() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = update_list_submitter_submit(
            ListSubmitterUpdatePath { submitter_id },
            context,
            list_submitter.clone(),
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
