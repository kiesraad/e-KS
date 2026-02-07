use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppEvent, AppStore, Context, HtmlTemplate, UtcDateTime,
    candidate_lists::{CandidateList, FullCandidateList},
    filters,
    form::FormData,
    persons::{COUNTRY_CODES, PersonForm},
};

use super::CreateCandidatePath;
#[derive(Template)]
#[template(path = "candidates/create.html")]
struct PersonCreateTemplate {
    full_list: FullCandidateList,
    form: FormData<PersonForm>,
    countries: &'static [&'static str],
}

pub async fn create_person_candidate_list(
    _: CreateCandidatePath,
    context: Context,
    full_list: FullCandidateList,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        PersonCreateTemplate {
            full_list,
            form: FormData::new(&context.csrf_tokens),
            countries: &COUNTRY_CODES,
        },
        context,
    )
    .into_response())
}

pub async fn create_person_candidate_list_submit(
    _: CreateCandidatePath,
    context: Context,
    full_list: FullCandidateList,
    State(store): State<AppStore>,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonCreateTemplate {
                full_list,
                form: form_data,
                countries: &COUNTRY_CODES,
            },
            context,
        )
        .into_response()),
        Ok(mut person) => {
            let now = UtcDateTime::now();
            person.created_at = now;
            person.updated_at = now;
            store.update(AppEvent::CreatePerson(person.clone())).await?;

            CandidateList::append_candidate(&store, full_list.id(), person.id).await?;

            let candidate = CandidateList::get_candidate(&store, full_list.id(), person.id).await?;

            Ok(Redirect::to(&candidate.after_create_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        test_utils::{response_body_string, sample_candidate_list, sample_person_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;

    #[tokio::test]
    async fn create_person_candidate_list_renders_form() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = create_person_candidate_list(
            CreateCandidatePath { list_id },
            Context::new_test_without_db(),
            full_list,
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.create_candidate_path()));
        assert!(body.contains("name=\"csrf_token\""));

        Ok(())
    }

    #[tokio::test]
    async fn create_person_candidate_list_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_person_form(&csrf_token);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = create_person_candidate_list_submit(
            CreateCandidatePath { list_id },
            context,
            full_list,
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

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        let candidate = full_list.candidates.first().expect("candidate");
        assert_eq!(location, candidate.after_create_path());

        Ok(())
    }

    #[tokio::test]
    async fn create_person_candidate_list_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = create_person_candidate_list_submit(
            CreateCandidatePath { list_id },
            context,
            full_list,
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
