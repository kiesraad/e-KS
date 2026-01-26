use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, AppResponse, Context, HtmlTemplate,
    candidate_lists::{
        Candidate, CandidateList, FullCandidateList, candidate_pages::CandidateListEditPersonPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, COUNTRY_CODES, PersonForm},
};

#[derive(Template)]
#[template(path = "candidates/update.html")]
struct PersonUpdateTemplate {
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<PersonForm>,
    countries: &'static [&'static str],
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
            countries: &COUNTRY_CODES,
        },
        context,
    ))
}

pub async fn update_person(
    _: CandidateListEditPersonPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    State(pool): State<PgPool>,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_update(candidate.person.clone(), &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                candidate,
                full_list,
                form: form_data,
                countries: &COUNTRY_CODES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::update_person(&pool, &person).await?;

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
        Context,
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

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&pool, list_id, person.id).await?;

        let response = edit_person_form(
            CandidateListEditPersonPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test(pool.clone()).await,
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

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&pool, list_id, person.id).await?;

        let context = Context::new_test(pool.clone()).await;
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
            State(pool.clone()),
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

        let updated = persons::get_person(&pool, person.id)
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

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&pool, list_id, person.id).await?;

        let context = Context::new_test(pool.clone()).await;
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
            State(pool.clone()),
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
