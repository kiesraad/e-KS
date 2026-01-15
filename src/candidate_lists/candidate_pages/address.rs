use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        candidate_pages::CandidateListEditAddressPath,
        pages::load_candidate_list,
        structs::{CandidateList, CandidateListEntry, FullCandidateList, MAX_CANDIDATES},
    },
    filters,
    form::{FormData, Validate},
    persons::{self, structs::AddressForm},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/address.html")]
struct PersonAddressUpdateTemplate {
    candidate: CandidateListEntry,
    form: FormData<AddressForm>,
    full_list: FullCandidateList,
    max_candidates: usize,
}

pub(crate) async fn edit_person_address(
    CandidateListEditAddressPath {
        candidate_list,
        person,
    }: CandidateListEditAddressPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> AppResponse<impl IntoResponse> {
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let candidate = full_list.get_candidate(&person, context.locale)?;
    let form = FormData::new_with_data(AddressForm::from(candidate.person.clone()), &csrf_tokens);

    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            form,
            candidate: candidate.clone(),
            full_list,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

pub(crate) async fn update_person_address(
    CandidateListEditAddressPath {
        candidate_list,
        person,
    }: CandidateListEditAddressPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<AddressForm>,
) -> Result<Response, AppError> {
    let full_list = load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let candidate = full_list.get_candidate(&person, context.locale)?;

    match form.validate(Some(&candidate.person), app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonAddressUpdateTemplate {
                candidate,
                form: form_data,
                full_list,
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
        extract::State,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, Locale, candidate_lists, persons,
        test_utils::{
            response_body_string, sample_address_form, sample_candidate_list,
            sample_person_with_last_name,
        },
    };

    #[sqlx::test]
    async fn edit_person_address_renders_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(Uuid::new_v4(), "Jansen");

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, &list_id, &[person.id])
            .await?;

        let response = edit_person_address(
            CandidateListEditAddressPath {
                candidate_list: list_id,
                person: person.id,
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
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(Uuid::new_v4(), "Jansen");

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, &list_id, &[person.id])
            .await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.locality = "Rotterdam".to_string();

        let response = update_person_address(
            CandidateListEditAddressPath {
                candidate_list: list_id,
                person: person.id,
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
        assert_eq!(location, list.view_path());

        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, &person.id)
            .await?
            .expect("updated person");
        assert_eq!(updated.locality, Some("Rotterdam".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(Uuid::new_v4(), "Jansen");

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, &list_id, &[person.id])
            .await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address(
            CandidateListEditAddressPath {
                candidate_list: list_id,
                person: person.id,
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
        assert!(body.contains("The value is too short"));

        Ok(())
    }
}
