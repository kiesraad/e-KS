use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        Candidate, CandidateList, FullCandidateList, candidate_pages::CandidateListEditAddressPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, AddressForm},
    t,
};

#[derive(Template)]
#[template(path = "candidates/edit_address.html")]
struct PersonAddressUpdateTemplate {
    candidate: Candidate,
    form: FormData<AddressForm>,
    full_list: FullCandidateList,
}

pub async fn edit_person_address(
    _: CandidateListEditAddressPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    full_list: FullCandidateList,
    candidate: Candidate,
) -> AppResponse<impl IntoResponse> {
    let form = FormData::new_with_data(AddressForm::from(candidate.person.clone()), &csrf_tokens);

    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            form,
            candidate: candidate.clone(),
            full_list,
        },
        context,
    ))
}

pub async fn update_person_address(
    _: CandidateListEditAddressPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    full_list: FullCandidateList,
    candidate: Candidate,
    DbConnection(mut conn): DbConnection,
    form: Form<AddressForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate.person, &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonAddressUpdateTemplate {
                candidate,
                form: form_data,
                full_list,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::update_address(&mut conn, &person).await?;

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
        persons::{self, PersonId},
        test_utils::{
            response_body_string, sample_address_form, sample_candidate_list,
            sample_person_with_last_name,
        },
    };

    #[sqlx::test]
    async fn edit_person_address_renders_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let response = edit_person_address(
            CandidateListEditAddressPath {
                list_id,
                person_id: person.id,
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
    async fn update_person_address_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.locality = "Rotterdam".to_string();

        let response = update_person_address(
            CandidateListEditAddressPath {
                list_id,
                person_id: person.id,
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
        let updated = persons::get_person(&mut conn, person.id)
            .await?
            .expect("updated person");
        assert_eq!(updated.locality, Some("Rotterdam".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address(
            CandidateListEditAddressPath {
                list_id,
                person_id: person.id,
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
        assert!(body.contains("The value is too short"));

        Ok(())
    }
}
