use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate, QueryParamState,
    candidate_lists::{CandidateList, ListSubmitterForm, pages::UpdateListSubmitterPath},
    filters,
    form::FormData,
    list_submitters::ListSubmitter,
    redirect_success,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/update_list_submitter.html")]
struct ListSubmitterUpdateTemplate {
    should_warn: bool,
    form: FormData<ListSubmitterForm>,
    candidate_list: CandidateList,
    list_submitters: Vec<ListSubmitter>,
    substitute_submitters: Vec<ListSubmitter>,
}

fn render_submitter_form(
    context: Context,
    candidate_list: CandidateList,
    store: &AppStore,
    should_warn: bool,
    form: FormData<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let list_submitters = store.get_list_submitters();
    let substitute_submitters = store.get_substitute_submitters();

    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            should_warn,
            form,
            candidate_list,
            list_submitters,
            substitute_submitters,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter(
    _: UpdateListSubmitterPath,
    context: Context,
    mut candidate_list: CandidateList,
    store: AppStore,
    Query(query): Query<QueryParamState>,
) -> Result<Response, AppError> {
    // When adding a new candidate list, select the default submitter and substitute submitters
    if query.is_initial() {
        candidate_list.select_default_submitters(&store)?;
    }

    let form = FormData::new_with_data(
        ListSubmitterForm::from(candidate_list.clone()),
        &context.session.csrf_tokens,
    );

    render_submitter_form(context, candidate_list, &store, query.should_warn(), form)
}

pub async fn update_list_submitter_submit(
    _: UpdateListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    store: AppStore,
    Query(query): Query<QueryParamState>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate_list, &context.session.csrf_tokens) {
        Err(form_data) => render_submitter_form(
            context,
            candidate_list,
            &store,
            query.should_warn(),
            form_data,
        ),
        Ok(candidate_list) => {
            candidate_list.update_submitters(&store).await?;

            Ok(redirect_success(candidate_list.view_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::Query,
        http::{StatusCode, header},
    };
    use axum_extra::routing::TypedPath;

    use crate::{
        AppStore, Context, ElectoralDistrict, Locale, QueryParamState, Session, TokenValue,
        candidate_lists::{CandidateListId, CandidateListSummary},
        list_submitters::ListSubmitterId,
        test_utils::{response_body_string, sample_candidate_list, sample_list_submitter},
    };

    #[tokio::test]
    async fn update_list_submitter_renders_submitter_form() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_list_submitter(ListSubmitterId::new());

        candidate_list.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let context = Context::new(&store, Session::new_with_locale(Locale::En));

        let response = update_list_submitter(
            UpdateListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            store,
            Query(QueryParamState::default()),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));
        assert!(body.contains("Substitute list submitters"));
        assert!(body.contains("csrf_token"));
        assert!(body.contains(&candidate_list.update_list_submitter_path().to_string()));
        assert!(body.contains(list_submitter.name.last_name.as_str()));
        assert!(body.contains(list_submitter.name.initials.as_str()));
        assert!(body.contains(substitute_submitter.name.last_name.as_str()));
        assert!(body.contains(substitute_submitter.name.initials.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitter_selects_defaults_on_initial_query() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let context = Context::new(&store, Session::new_with_locale(Locale::En));
        let candidate_list = sample_candidate_list(CandidateListId::new());

        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter_a = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter_b = sample_list_submitter(ListSubmitterId::new());

        candidate_list.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter_a.create_substitute(&store).await?;
        substitute_submitter_b.create_substitute(&store).await?;

        let response = update_list_submitter(
            UpdateListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            store,
            Query(QueryParamState::created()),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&format!("value=\"{}\" checked", list_submitter.id)));
        assert!(body.contains(&format!("value=\"{}\" checked", substitute_submitter_a.id)));
        assert!(body.contains(&format!("value=\"{}\" checked", substitute_submitter_b.id)));

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitters_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new(&store, Session::new_with_locale(Locale::En));
        let csrf_token = context.session.csrf_tokens.issue().value;
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            ..Default::default()
        };
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter_a = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter_b = sample_list_submitter(ListSubmitterId::new());

        candidate_list.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter_a.create(&store).await?;
        substitute_submitter_b.create(&store).await?;

        let form = ListSubmitterForm {
            list_submitter_id: list_submitter.id.to_string(),
            substitute_list_submitter_ids: vec![
                substitute_submitter_a.id,
                substitute_submitter_b.id,
            ],
            csrf_token,
        };
        let response = update_list_submitter_submit(
            UpdateListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            store.clone(),
            Query(QueryParamState::default()),
            Form(form),
        )
        .await
        .unwrap();

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        // verify updated candidate list object in database
        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        assert_eq!(
            updated_list
                .view_path()
                .with_query_params(QueryParamState::success())
                .to_string(),
            location
        );

        assert_eq!(candidate_list.id, updated_list.id);

        assert_eq!(list_submitter.id, updated_list.list_submitter_id.unwrap());
        assert_eq!(
            vec![substitute_submitter_a.id, substitute_submitter_b.id],
            updated_list.substitute_list_submitter_ids
        );

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitters_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new(&store, Session::new_with_locale(Locale::En));
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_list_submitter(ListSubmitterId::new());

        candidate_list.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let form = ListSubmitterForm {
            list_submitter_id: list_submitter.id.to_string(),
            substitute_list_submitter_ids: vec![substitute_submitter.id],
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update_list_submitter_submit(
            UpdateListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            store.clone(),
            Query(QueryParamState::default()),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));
        assert!(body.contains("Substitute list submitters"));

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        // verify candidate list didn't update in database
        assert!(updated_list.list_submitter_id.is_none());
        assert!(updated_list.substitute_list_submitter_ids.is_empty());

        Ok(())
    }
}
