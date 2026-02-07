use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, InitialQuery,
    candidate_lists::{
        CandidateList, ListSubmitterForm,
        pages::{UpdateListSubmitterPath, UpdateSubstituteListSubmittersPath},
    },
    filters,
    form::FormData,
    list_submitters::ListSubmitter,
    substitute_list_submitters::SubstituteSubmitter,
};

#[derive(Template)]
#[template(path = "candidate_lists/list_submitter.html")]
struct ListSubmitterUpdateTemplate {
    should_warn: bool,
    form: FormData<ListSubmitterForm>,
    candidate_list: CandidateList,
    list_submitters: Vec<ListSubmitter>,
    substitute_submitters: Vec<SubstituteSubmitter>,
    form_action: String,
}

fn render_submitter_form(
    context: Context,
    candidate_list: CandidateList,
    store: &AppStore,
    should_warn: bool,
    form: FormData<ListSubmitterForm>,
    form_action: String,
) -> Result<Response, AppError> {
    let list_submitters = store.get_list_submitters()?;
    let substitute_submitters = store.get_substitute_submitters()?;

    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            should_warn,
            form,
            candidate_list,
            list_submitters,
            substitute_submitters,
            form_action,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter(
    _: UpdateListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
    Query(query): Query<InitialQuery>,
) -> Result<Response, AppError> {
    let form = FormData::new_with_data(
        ListSubmitterForm::from(candidate_list.clone()),
        &context.csrf_tokens,
    );
    let form_action = candidate_list.update_list_submitter_path();

    render_submitter_form(
        context,
        candidate_list,
        &store,
        query.should_warn(),
        form,
        form_action,
    )
}

pub async fn update_substitute_list_submitters(
    _: UpdateSubstituteListSubmittersPath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
    Query(query): Query<InitialQuery>,
) -> Result<Response, AppError> {
    let form = FormData::new_with_data(
        ListSubmitterForm::from(candidate_list.clone()),
        &context.csrf_tokens,
    );
    let form_action = candidate_list.update_substitute_list_submitters_path();

    render_submitter_form(
        context,
        candidate_list,
        &store,
        query.should_warn(),
        form,
        form_action,
    )
}

pub async fn update_list_submitter_submit(
    _: UpdateListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
    Query(query): Query<InitialQuery>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let form_action = candidate_list.update_list_submitter_path();

    match form.validate_update(&candidate_list, &context.csrf_tokens) {
        Err(form_data) => render_submitter_form(
            context,
            candidate_list,
            &store,
            query.should_warn(),
            form_data,
            form_action,
        ),
        Ok(candidate_list) => {
            candidate_list.update(&store).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}

pub async fn update_substitute_list_submitters_submit(
    _: UpdateSubstituteListSubmittersPath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
    Query(query): Query<InitialQuery>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let form_action = candidate_list.update_substitute_list_submitters_path();

    match form.validate_update(&candidate_list, &context.csrf_tokens) {
        Err(form_data) => render_submitter_form(
            context,
            candidate_list,
            &store,
            query.should_warn(),
            form_data,
            form_action,
        ),
        Ok(candidate_list) => {
            candidate_list.update(&store).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
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
    use axum_extra::extract::Form;

    use crate::{
        AppStore, Context, CsrfTokens, ElectoralDistrict, InitialQuery, Locale, TokenValue,
        UtcDateTime,
        candidate_lists::{CandidateListId, CandidateListSummary},
        list_submitters::ListSubmitterId,
        political_groups::PoliticalGroupId,
        substitute_list_submitters::SubstituteSubmitterId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_list_submitter,
            sample_political_group, sample_substitute_submitter,
        },
    };

    #[tokio::test]
    async fn update_list_submitter_renders_submitter_form() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_substitute_submitter(SubstituteSubmitterId::new());
        let political_group = sample_political_group(PoliticalGroupId::new());

        candidate_list.create(&store).await?;
        political_group.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());

        let response = update_list_submitter(
            UpdateListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store),
            Query(InitialQuery::default()),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));
        assert!(body.contains("Substitute list submitters"));
        assert!(body.contains("csrf_token"));
        assert!(body.contains(&candidate_list.update_list_submitter_path()));
        assert!(body.contains(list_submitter.name.last_name.as_str()));
        assert!(body.contains(list_submitter.name.initials.as_str()));
        assert!(body.contains(substitute_submitter.name.last_name.as_str()));
        assert!(body.contains(substitute_submitter.name.initials.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn update_substitute_list_submitters_renders_submitter_form() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_substitute_submitter(SubstituteSubmitterId::new());
        let political_group = sample_political_group(PoliticalGroupId::new());

        candidate_list.create(&store).await?;
        political_group.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());

        let response = update_substitute_list_submitters(
            UpdateSubstituteListSubmittersPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store),
            Query(InitialQuery::default()),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));
        assert!(body.contains("Substitute list submitters"));
        assert!(body.contains("csrf_token"));
        assert!(body.contains(&candidate_list.update_substitute_list_submitters_path()));
        assert!(body.contains(list_submitter.name.last_name.as_str()));
        assert!(body.contains(list_submitter.name.initials.as_str()));
        assert!(body.contains(substitute_submitter.name.last_name.as_str()));
        assert!(body.contains(substitute_submitter.name.initials.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn update_substitute_list_submitters_shows_warnings_when_not_initial()
    -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_substitute_submitter(SubstituteSubmitterId::new());
        let political_group = sample_political_group(PoliticalGroupId::new());

        candidate_list.create(&store).await?;
        political_group.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());
        let query: InitialQuery =
            serde_urlencoded::from_str("initial=false").expect("initial query");

        let response = update_substitute_list_submitters(
            UpdateSubstituteListSubmittersPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store),
            Query(query),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&format!(
            "href=\"{}\" class=\"ok\"",
            candidate_list.update_path()
        )));
        assert!(body.contains(&format!(
            "href=\"{}\" class=\"warning\"",
            candidate_list.update_list_submitter_path()
        )));

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitters_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let political_group = sample_political_group(PoliticalGroupId::new());
        political_group.create(&store).await?;
        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());
        let csrf_token = context.csrf_tokens.issue().value;
        let creation_date = UtcDateTime::now();
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
            ..Default::default()
        };
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter_a = sample_substitute_submitter(SubstituteSubmitterId::new());
        let substitute_submitter_b = sample_substitute_submitter(SubstituteSubmitterId::new());

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
            State(store.clone()),
            Query(InitialQuery::default()),
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

        assert_eq!(updated_list.view_path(), location);

        assert_eq!(candidate_list.id, updated_list.id);

        assert_eq!(list_submitter.id, updated_list.list_submitter_id.unwrap());
        assert_eq!(
            vec![substitute_submitter_a.id, substitute_submitter_b.id],
            updated_list.substitute_list_submitter_ids
        );

        Ok(())
    }

    #[tokio::test]
    async fn update_substitute_list_submitters_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let political_group = sample_political_group(PoliticalGroupId::new());
        political_group.create(&store).await?;
        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());
        let csrf_token = context.csrf_tokens.issue().value;
        let creation_date = UtcDateTime::now();
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
            ..Default::default()
        };
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter_a = sample_substitute_submitter(SubstituteSubmitterId::new());
        let substitute_submitter_b = sample_substitute_submitter(SubstituteSubmitterId::new());

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
        let response = update_substitute_list_submitters_submit(
            UpdateSubstituteListSubmittersPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store.clone()),
            Query(InitialQuery::default()),
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

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        assert_eq!(updated_list.view_path(), location);
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
        let store = AppStore::new_for_test().await;
        let political_group = sample_political_group(PoliticalGroupId::new());
        political_group.create(&store).await?;
        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_substitute_submitter(SubstituteSubmitterId::new());

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
            State(store.clone()),
            Query(InitialQuery::default()),
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

    #[tokio::test]
    async fn update_substitute_list_submitters_invalid_form_renders_template()
    -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let political_group = sample_political_group(PoliticalGroupId::new());
        political_group.create(&store).await?;
        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_substitute_submitter(SubstituteSubmitterId::new());

        candidate_list.create(&store).await?;
        list_submitter.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let form = ListSubmitterForm {
            list_submitter_id: list_submitter.id.to_string(),
            substitute_list_submitter_ids: vec![substitute_submitter.id],
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update_substitute_list_submitters_submit(
            UpdateSubstituteListSubmittersPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store.clone()),
            Query(InitialQuery::default()),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));
        assert!(body.contains("Substitute list submitters"));
        assert!(body.contains(&candidate_list.update_substitute_list_submitters_path()));

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        // verify candidate list didn't update in database
        assert!(updated_list.list_submitter_id.is_none());
        assert!(updated_list.substitute_list_submitter_ids.is_empty());

        Ok(())
    }
}
