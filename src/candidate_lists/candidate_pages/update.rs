use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        Candidate, CandidateList, FullCandidateList, MAX_CANDIDATES,
        candidate_pages::CandidateListEditPersonPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, PersonForm},
    t,
};

#[derive(Template)]
#[template(path = "candidates/update.html")]
struct PersonUpdateTemplate {
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<PersonForm>,
    max_candidates: usize,
}

pub async fn edit_person_form(
    CandidateListEditPersonPath {
        candidate_list: _,
        person: _,
    }: CandidateListEditPersonPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    full_list: FullCandidateList,
    candidate: Candidate,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(PersonForm::from(candidate.person.clone()), &csrf_tokens),
            candidate,
            full_list,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

pub async fn update_person(
    CandidateListEditPersonPath {
        candidate_list: _,
        person: _,
    }: CandidateListEditPersonPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    full_list: FullCandidateList,
    candidate: Candidate,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate.person, &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                candidate,
                full_list,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::repository::update_person(&mut conn, &person).await?;

            Ok(Redirect::to(&full_list.list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        Context, CsrfTokens, DbConnection, Locale,
        candidate_lists::{self, CandidateListId},
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person, sample_person_form,
        },
    };

    #[sqlx::test]
    async fn edit_person_form_renders_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, list_id, &[person.id])
            .await?;

        let full_list = candidate_lists::repository::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate =
            candidate_lists::repository::get_candidate(&mut conn, list_id, person.id).await?;

        let response = edit_person_form(
            CandidateListEditPersonPath {
                candidate_list: list_id,
                person: person.id,
            },
            Context::new(Locale::En),
            CsrfTokens::default(),
            full_list,
            candidate,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, list_id, &[person.id])
            .await?;

        let full_list = candidate_lists::repository::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate =
            candidate_lists::repository::get_candidate(&mut conn, list_id, person.id).await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_person(
            CandidateListEditPersonPath {
                candidate_list: list_id,
                person: person.id,
            },
            Context::new(Locale::En),
            csrf_tokens,
            full_list,
            candidate,
            DbConnection(pool.acquire().await?),
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

        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, person.id)
            .await?
            .expect("updated person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_invalid_form_renders_template(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, list_id, &[person.id])
            .await?;

        let full_list = candidate_lists::repository::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate =
            candidate_lists::repository::get_candidate(&mut conn, list_id, person.id).await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_person(
            CandidateListEditPersonPath {
                candidate_list: list_id,
                person: person.id,
            },
            Context::new(Locale::En),
            csrf_tokens,
            full_list,
            candidate,
            DbConnection(pool.acquire().await?),
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
