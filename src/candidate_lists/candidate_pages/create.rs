use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context, HtmlTemplate,
    candidate_lists::{self, CandidateList, FullCandidateList, pages::CreateCandidatePath},
    filters,
    form::{FormData, Validate},
    persons::{self, COUNTRY_CODES, PersonForm},
};

#[derive(Template)]
#[template(path = "candidates/create.html")]
struct PersonCreateTemplate {
    full_list: FullCandidateList,
    form: FormData<PersonForm>,
    countries: &'static [&'static str],
}

pub async fn new_person_candidate_list(
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

pub async fn create_person_candidate_list(
    _: CreateCandidatePath,
    context: Context,
    full_list: FullCandidateList,
    State(pool): State<PgPool>,
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
        Ok(person) => {
            let person = persons::create_person(&pool, &person).await?;

            candidate_lists::append_candidate_to_list(&pool, full_list.id(), person.id).await?;

            let candidate =
                candidate_lists::get_candidate(&pool, full_list.id(), person.id).await?;

            Ok(Redirect::to(&candidate.edit_address_path()).into_response())
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
        test_utils::{response_body_string, sample_candidate_list, sample_person_form},
    };

    #[sqlx::test]
    async fn new_person_candidate_list_renders_form(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        candidate_lists::create_candidate_list(&pool, &list).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");

        let response = new_person_candidate_list(
            CreateCandidatePath { list_id },
            Context::new_test(pool.clone()).await,
            full_list,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.new_candidate_path()));
        assert!(body.contains("name=\"csrf_token\""));

        Ok(())
    }

    #[sqlx::test]
    async fn create_person_candidate_list_persists_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        candidate_lists::create_candidate_list(&pool, &list).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_person_form(&csrf_token);

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");

        let response = create_person_candidate_list(
            CreateCandidatePath { list_id },
            context,
            full_list,
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

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        let candidate = full_list.candidates.first().expect("candidate");
        assert_eq!(location, candidate.edit_address_path());

        Ok(())
    }

    #[sqlx::test]
    async fn create_person_candidate_list_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        candidate_lists::create_candidate_list(&pool, &list).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");

        let response = create_person_candidate_list(
            CreateCandidatePath { list_id },
            context,
            full_list,
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
