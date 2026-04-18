use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};

use crate::{
    AppError, AppStore, Context, ElectoralDistrict, Form, HtmlTemplate,
    candidate_lists::{
        CandidateList, CandidateListCreateForm, CandidateListId, pages::CandidateListCreatePath,
    },
    filters,
    form::FormData,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/create.html")]
struct CandidateListCreateTemplate {
    form: FormData<CandidateListCreateForm>,
    available_districts: Vec<ElectoralDistrict>,
    has_previous_list: bool,
}

pub async fn create_candidate_list(
    _: CandidateListCreatePath,
    context: Context,
    store: AppStore,
) -> Result<Response, AppError> {
    if context.election.has_only_one_district() {
        if !store.get_candidate_lists().is_empty() {
            return Err(AppError::UserError(
                "Cannot create more than one candidate list for single district elections"
                    .to_string(),
            ));
        }
        let list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: context.election.electoral_districts().to_vec(),
            ..Default::default()
        };
        list.create(&store).await?;
        return Ok(Redirect::to(&list.after_create_path().to_string()).into_response());
    }

    let available_districts = CandidateList::available_districts(&store, &context.election);
    let has_previous_list = !store.get_candidate_lists().is_empty();
    Ok(HtmlTemplate(
        CandidateListCreateTemplate {
            form: FormData::new(&context.session.csrf_tokens),
            available_districts,
            has_previous_list,
        },
        context,
    )
    .into_response())
}

pub async fn create_candidate_list_submit(
    _: CandidateListCreatePath,
    context: Context,
    store: AppStore,
    Form(mut form): Form<CandidateListCreateForm>,
) -> Result<Response, AppError> {
    if context.election.has_only_one_district() {
        return Err(AppError::UserError(
            "Not available for single district elections".to_string(),
        ));
    }
    let available_districts = CandidateList::available_districts(&store, &context.election);
    let should_copy_candidates = form.copy_candidates;
    form.electoral_districts
        .retain(|district| context.election.electoral_districts().contains(district));

    match form.validate_create(&context.session.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            CandidateListCreateTemplate {
                form: form_data,
                has_previous_list: !store.get_candidate_lists().is_empty(),
                available_districts,
            },
            context,
        )
        .into_response()),
        Ok(mut candidate_list) => {
            if should_copy_candidates {
                candidate_list.candidates = store
                    .get_candidate_lists()
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
        AppStore, Context, ElectionConfig, ElectoralDistrict, Locale, Province, Session,
        TokenValue, WaterCouncil,
        candidate_lists::{CandidateListId, CandidateListSummary},
        persons::PersonId,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[tokio::test]
    async fn create_candidate_list_renders_csrf_field() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let response = create_candidate_list(
            CandidateListCreatePath {},
            Context::new_test_without_db(),
            store,
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
        let store = AppStore::new_for_test();
        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = CandidateListCreateForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            copy_candidates: false,
            csrf_token,
        };

        let response = create_candidate_list_submit(
            CandidateListCreatePath {},
            context,
            store.clone(),
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
        let store = AppStore::new_for_test();
        let form = CandidateListCreateForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            copy_candidates: false,
            csrf_token: TokenValue("invalid".to_string()),
        };

        let response = create_candidate_list_submit(
            CandidateListCreatePath {},
            Context::new_test_without_db(),
            store,
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
        let store = AppStore::new_for_test();
        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
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
            store.clone(),
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
        let election = ElectionConfig::EK27;
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

    #[tokio::test]
    async fn create_candidate_list_with_district_election_persists() -> Result<(), AppError> {
        let store =
            AppStore::new_for_test_with_election(ElectionConfig::WS27(WaterCouncil::Fryslan));
        let context = Context::new(
            &store,
            Session::new_with_locale(&"test_id_code".into(), Locale::En),
        );

        let response =
            create_candidate_list(CandidateListCreatePath {}, context, store.clone()).await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);
        assert_eq!(
            lists[0].list.electoral_districts,
            vec![ElectoralDistrict::WsFryslan]
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_candidate_list_with_provincial_election_persists() -> Result<(), AppError> {
        let store = AppStore::new_for_test_with_election(ElectionConfig::PS27(Province::GE));
        let context = Context::new(&store, Session::new_test_with_locale(Locale::En));
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = CandidateListCreateForm {
            electoral_districts: vec![ElectoralDistrict::PsNijmegen],
            copy_candidates: false,
            csrf_token,
        };

        let response = create_candidate_list_submit(
            CandidateListCreatePath {},
            context,
            store.clone(),
            Form(form),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);
        assert_eq!(
            lists[0].list.electoral_districts,
            vec![ElectoralDistrict::PsNijmegen]
        );

        Ok(())
    }

    #[tokio::test]
    async fn single_district_election_blocks_2nd_create_on_get() -> Result<(), AppError> {
        let store =
            AppStore::new_for_test_with_election(ElectionConfig::WS27(WaterCouncil::AaEnMaas));

        let mut context = Context::new(
            &store,
            Session::new_with_locale(&"test_id_code".into(), Locale::En),
        );
        context.election = ElectionConfig::WS27(WaterCouncil::AaEnMaas); // select election with only one district
        sample_candidate_list(CandidateListId::new())
            .create(&store)
            .await?;

        // test
        let error = create_candidate_list(CandidateListCreatePath {}, context, store.clone())
            .await
            .err()
            .unwrap();

        // verify
        match error {
            AppError::UserError(msg) => assert_eq!(
                msg,
                "Cannot create more than one candidate list for single district elections"
                    .to_string()
            ),
            _ => panic!("should be user error"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn single_district_election_blocks_create_on_post() -> Result<(), AppError> {
        let store =
            AppStore::new_for_test_with_election(ElectionConfig::WS27(WaterCouncil::AaEnMaas));

        let mut context = Context::new(
            &store,
            Session::new_with_locale(&"test_id_code".into(), Locale::En),
        );
        context.election = ElectionConfig::WS27(WaterCouncil::AaEnMaas); // select election with only one district

        // test
        let error = create_candidate_list_submit(
            CandidateListCreatePath {},
            context,
            store,
            Form(CandidateListCreateForm {
                ..Default::default()
            }),
        )
        .await
        .err()
        .unwrap();

        // verify
        match error {
            AppError::UserError(msg) => assert_eq!(
                msg,
                "Not available for single district elections".to_string()
            ),
            _ => panic!("should be user error"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn district_outside_election_is_ignored() -> Result<(), AppError> {
        // setup
        let store = AppStore::new_for_test_with_election(ElectionConfig::EK27);
        let mut context = Context::new(
            &store,
            Session::new_with_locale(&"test_id_code".into(), Locale::En),
        );
        context.election = ElectionConfig::EK27;
        let csrf_token = context.session.csrf_tokens.issue().value;

        // test
        let response = create_candidate_list_submit(
            CandidateListCreatePath {},
            context,
            store.clone(),
            Form(CandidateListCreateForm {
                electoral_districts: vec![ElectoralDistrict::WsFryslan, ElectoralDistrict::UT],
                copy_candidates: false,
                csrf_token,
            }),
        )
        .await?;

        // verify
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let lists = store.get_candidate_lists();
        assert_eq!(lists.len(), 1);
        let list = &lists[0];
        // WsFryslan got dropped because it's not part of EK27
        assert_eq!(list.electoral_districts, vec![ElectoralDistrict::UT]);

        Ok(())
    }
}
