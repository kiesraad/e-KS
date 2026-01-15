use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        self, CandidateList, FullCandidateList, MAX_CANDIDATES,
        pages::{CreateCandidatePath, load_candidate_list},
    },
    filters,
    form::{FormData, Validate},
    persons::{self, PersonForm},
    t,
};

#[derive(Template)]
#[template(path = "candidates/create.html")]
struct PersonCreateTemplate {
    full_list: FullCandidateList,
    form: FormData<PersonForm>,
    max_candidates: usize,
}

pub async fn new_person_candidate_list(
    CreateCandidatePath { candidate_list }: CreateCandidatePath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    Ok(HtmlTemplate(
        PersonCreateTemplate {
            full_list,
            form: FormData::new(&csrf_tokens),
            max_candidates: MAX_CANDIDATES,
        },
        context,
    )
    .into_response())
}

pub async fn create_person_candidate_list(
    CreateCandidatePath { candidate_list }: CreateCandidatePath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    match form.validate(None, app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonCreateTemplate {
                full_list,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            let person = persons::repository::create_person(&mut conn, &person).await?;

            let mut person_ids = full_list.get_ids();
            person_ids.push(person.id);
            candidate_lists::repository::update_candidate_list_order(
                &mut conn,
                &candidate_list,
                &person_ids,
            )
            .await?;

            let candidate =
                candidate_lists::repository::get_candidate(&mut conn, &candidate_list, &person.id)
                    .await?;

            Ok(Redirect::to(&candidate.edit_address_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::State,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, Locale, candidate_lists,
        test_utils::{response_body_string, sample_candidate_list, sample_person_form},
    };

    #[sqlx::test]
    async fn new_person_candidate_list_renders_form(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let mut conn = pool.acquire().await?;

        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;

        let response = new_person_candidate_list(
            CreateCandidatePath {
                candidate_list: list_id,
            },
            Context::new(Locale::En),
            CsrfTokens::default(),
            DbConnection(pool.acquire().await?),
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
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let form = sample_person_form(&csrf_token);

        let response = create_person_candidate_list(
            CreateCandidatePath {
                candidate_list: list_id,
            },
            Context::new(Locale::En),
            State(app_state),
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

        let mut conn = pool.acquire().await?;
        let full_list = load_candidate_list(&mut conn, &list_id, Locale::En)
            .await
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
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_person_candidate_list(
            CreateCandidatePath {
                candidate_list: list_id,
            },
            Context::new(Locale::En),
            State(app_state),
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
