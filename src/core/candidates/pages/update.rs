use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate,
    candidate_lists::FullCandidateList, candidates::Candidate, filters, form::FormData,
    persons::PersonForm,
};

use super::CandidateListUpdatePersonPath;
#[derive(Template)]
#[template(path = "candidates/update.html")]
struct PersonUpdateTemplate {
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<PersonForm>,
}

pub async fn update_person(
    _: CandidateListUpdatePersonPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(
                PersonForm::from(candidate.person.clone()),
                &context.csrf_tokens,
            ),
            candidate,
            full_list,
        },
        context,
    ))
}

pub async fn update_person_submit(
    _: CandidateListUpdatePersonPath,
    context: Context,
    full_list: FullCandidateList,
    mut candidate: Candidate,
    State(store): State<AppStore>,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate.person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                candidate,
                full_list,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            person.update(&store).await?;
            candidate.person = person;

            Ok(Redirect::to(&candidate.after_update_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context, Form,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person, sample_person_form,
        },
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };

    #[tokio::test]
    async fn update_person_renders_candidate() -> Result<(), AppError> {
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

        let response = update_person(
            CandidateListUpdatePersonPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test_without_db(),
            full_list,
            candidate,
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn update_person_persists_and_redirects() -> Result<(), AppError> {
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
        let mut form = sample_person_form(&csrf_token);
        form.name.last_name = "Updated".to_string();
        let expected_path = candidate.after_update_path();

        let response = update_person_submit(
            CandidateListUpdatePersonPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
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
        assert_eq!(location, expected_path);

        let updated = store
            .get_persons()?
            .into_iter()
            .find(|p| p.id == person.id)
            .expect("updated person");
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_person_invalid_form_renders_template() -> Result<(), AppError> {
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
        let mut form = sample_person_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = update_person_submit(
            CandidateListUpdatePersonPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(store),
            Form(form),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}
