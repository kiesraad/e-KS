use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, Context, DbConnection, HtmlTemplate,
    candidate_lists::{
        Candidate, CandidateList, FullCandidateList, candidate_pages::CandidateListEditPersonPath,
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
}

pub async fn edit_person_form(
    _: CandidateListEditPersonPath,
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

pub async fn update_person(
    _: CandidateListEditPersonPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
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
            persons::update_person(&mut conn, &person).await?;

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
        Context, DbConnection,
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
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let response = edit_person_form(
            CandidateListEditPersonPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test(),
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
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_person(
            CandidateListEditPersonPath {
                list_id,
                person_id: person.id,
            },
            context,
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
        let updated = persons::get_person(&mut conn, person.id)
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
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_person(
            CandidateListEditPersonPath {
                list_id,
                person_id: person.id,
            },
            context,
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
