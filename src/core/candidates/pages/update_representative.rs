use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate,
    candidate_lists::FullCandidateList,
    candidates::Candidate,
    filters,
    form::FormData,
    persons::{InitialQuery, RepresentativeForm},
};

use super::UpdateRepresentativePath;
#[derive(Template)]
#[template(path = "candidates/update_representative.html")]
struct UpdateRepresentativeTemplate {
    should_warn: bool,
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<RepresentativeForm>,
}

pub async fn update_representative(
    _: UpdateRepresentativePath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    Query(query): Query<InitialQuery>,
) -> AppResponse<impl IntoResponse> {
    let form = FormData::new_with_data(
        RepresentativeForm::from(candidate.person.representative.clone()),
        &context.csrf_tokens,
    );

    Ok(HtmlTemplate(
        UpdateRepresentativeTemplate {
            should_warn: query.should_warn(),
            candidate: candidate.clone(),
            full_list,
            form,
        },
        context,
    ))
}

pub async fn update_representative_submit(
    _: UpdateRepresentativePath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    State(store): State<AppStore>,
    Query(query): Query<InitialQuery>,
    Form(form): Form<RepresentativeForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate.person.representative, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            UpdateRepresentativeTemplate {
                should_warn: query.should_warn(),
                candidate,
                full_list,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(representative) => {
            candidate
                .person
                .update_representative(&store, representative)
                .await?;

            Ok(Redirect::to(&full_list.list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{
            extract_csrf_token, response_body_string, sample_candidate_list, sample_person,
            sample_representative_form,
        },
    };
    use axum::{
        extract::Query,
        http::{StatusCode, header},
        response::IntoResponse,
    };

    #[tokio::test]
    async fn update_representative_renders_candidate() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        person.create(&store).await?;
        list.clone().update_order(&store, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person.id)
            .await?;

        let response = update_representative(
            UpdateRepresentativePath {
                list_id,
                person_id: person.id,
            },
            Context::new_test_without_db(),
            full_list,
            candidate,
            Query(InitialQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn update_representative_renders_valid_csrf_token() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        person.create(&store).await?;
        list.clone().update_order(&store, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_tokens = context.csrf_tokens.clone();

        let response = update_representative(
            UpdateRepresentativePath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            Query(InitialQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        let csrf_token = extract_csrf_token(&body).expect("csrf token");
        assert!(csrf_tokens.consume(&csrf_token));

        Ok(())
    }

    #[tokio::test]
    async fn update_representative_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        person.create(&store).await?;
        list.clone().update_order(&store, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_representative_form(&csrf_token);
        form.name.last_name = "Smit".to_string();

        let response = update_representative_submit(
            UpdateRepresentativePath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
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
        assert_eq!(location, list.view_path());

        let updated = store.get_person(person.id)?;
        assert_eq!(updated.representative.name.last_name.to_string(), "Smit");

        Ok(())
    }

    #[tokio::test]
    async fn update_representative_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        person.create(&store).await?;
        list.clone().update_order(&store, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_representative_form(&csrf_token);
        form.address.postal_code = "a".to_string();

        let response = update_representative_submit(
            UpdateRepresentativePath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(store),
            Query(InitialQuery::default()),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The postal code is not valid"));

        Ok(())
    }
}
