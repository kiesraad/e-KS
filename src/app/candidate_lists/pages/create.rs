use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, Context, ElectionConfig, Form, HtmlTemplate, Store,
    candidate_lists::{CandidateList, CandidateListCreateForm, pages::CandidateListCreatePath},
    filters,
    form::FormData,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/create.html")]
struct CandidateListCreateTemplate {
    form: FormData<CandidateListCreateForm>,
    has_previous_list: bool,
}

pub async fn create_candidate_list(
    _: CandidateListCreatePath,
    context: Context,
    State(store): State<Store>,
) -> Result<impl IntoResponse, AppError> {
    let used_districts = CandidateList::used_districts(&store, vec![])?;
    let available_districts = context.election.available_districts(used_districts);
    let has_previous_list = !store.get_candidate_lists()?.is_empty();

    let form = FormData::new_with_data(
        CandidateListCreateForm {
            electoral_districts: available_districts,
            ..Default::default()
        },
        &context.csrf_tokens,
    );

    Ok(HtmlTemplate(
        CandidateListCreateTemplate {
            form,
            has_previous_list,
        },
        context,
    )
    .into_response())
}

pub async fn create_candidate_list_submit(
    _: CandidateListCreatePath,
    context: Context,
    State(store): State<Store>,
    Form(form): Form<CandidateListCreateForm>,
) -> Result<Response, AppError> {
    let should_copy_candidates = form.copy_candidates;
    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            CandidateListCreateTemplate {
                form: form_data,
                has_previous_list: !store.get_candidate_lists()?.is_empty(),
            },
            context,
        )
        .into_response()),
        Ok(mut candidate_list) => {
            if should_copy_candidates {
                candidate_list.candidates = store
                    .get_candidate_lists()?
                    .last()
                    .map(|list| list.candidates.clone())
                    .unwrap_or_default();
            }

            candidate_list.create(&store).await?;

            Ok(Redirect::to(&candidate_list.after_create_path().to_string()).into_response())
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::*;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };

    use crate::{
        Context, ElectoralDistrict, Store, TokenValue,
        candidate_lists::{CandidateListId, CandidateListSummary},
        persons::PersonId,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[tokio::test]
    async fn create_candidate_list_renders_csrf_field() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let response = create_candidate_list(
            CandidateListCreatePath {},
            Context::new_test_without_db(),
            State(store),
        )
        .await?
        .into_response();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));

        Ok(())
    }

    #[tokio::test]
    async fn create_candidate_list_persists_and_redirects() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = CandidateListCreateForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            copy_candidates: false,
            csrf_token,
        };

        let response = create_candidate_list_submit(
            CandidateListCreatePath {},
            context,
            State(store.clone()),
            Form(form),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);

        let expected = lists[0].list.after_create_path().to_string();
        assert_eq!(location, expected);

        Ok(())
    }

    #[tokio::test]
    async fn create_candidate_list_invalid_form_renders_template() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let form = CandidateListCreateForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            copy_candidates: false,
            csrf_token: TokenValue("invalid".to_string()),
        };

        let response = create_candidate_list_submit(
            CandidateListCreatePath {},
            Context::new_test_without_db(),
            State(store),
            Form(form),
        )
        .await?;

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Create candidate list"));

        Ok(())
    }

    #[tokio::test]
    async fn create_candidate_list_copies_previous_candidates() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let person_a = sample_person(PersonId::new());
        let person_b = sample_person(PersonId::new());

        person_a.create(&store).await?;
        person_b.create(&store).await?;
        list.candidates = vec![person_a.id, person_b.id];
        list.create(&store).await?;

        let form = CandidateListCreateForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            copy_candidates: true,
            csrf_token,
        };

        create_candidate_list_submit(
            CandidateListCreatePath {},
            context,
            State(store.clone()),
            Form(form),
        )
        .await?;

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 2);
        let new_list = &lists[1].list;
        assert_eq!(new_list.candidates, vec![person_a.id, person_b.id]);

        Ok(())
    }

    #[tokio::test]
    async fn test_determine_available_districts() {
        let election = ElectionConfig::EK2027;
        let all_districts = election.electoral_districts().to_vec();

        let none_used = vec![];
        let all_used = all_districts.clone();
        let some_used = vec![
            ElectoralDistrict::DR,
            ElectoralDistrict::FL,
            ElectoralDistrict::FR,
            ElectoralDistrict::GE,
            ElectoralDistrict::GR,
            ElectoralDistrict::LI,
            ElectoralDistrict::NB,
            ElectoralDistrict::NH,
        ];

        // use sets so we don't need to worry about ordering of the vector
        let none_used_result: BTreeSet<ElectoralDistrict> = election
            .available_districts(none_used)
            .into_iter()
            .collect();
        let all_used_result: BTreeSet<ElectoralDistrict> =
            election.available_districts(all_used).into_iter().collect();
        let some_used_result: BTreeSet<ElectoralDistrict> = election
            .available_districts(some_used)
            .into_iter()
            .collect();

        // validation
        let all_district_set: BTreeSet<ElectoralDistrict> = all_districts.into_iter().collect();
        assert_eq!(all_district_set, none_used_result);
        assert_eq!(BTreeSet::new(), all_used_result);
        assert_eq!(
            BTreeSet::from([
                ElectoralDistrict::OV,
                ElectoralDistrict::UT,
                ElectoralDistrict::ZE,
                ElectoralDistrict::ZH,
                ElectoralDistrict::BO,
                ElectoralDistrict::SE,
                ElectoralDistrict::SA,
                ElectoralDistrict::KN,
            ]),
            some_used_result
        );
    }
}
