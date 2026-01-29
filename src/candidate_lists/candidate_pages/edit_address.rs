use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, AppResponse, Context, HtmlTemplate,
    candidate_lists::{
        Candidate, CandidateList, FullCandidateList, candidate_pages::CandidateListEditAddressPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, AddressForm, InitialEditQuery},
};

#[derive(Template)]
#[template(path = "candidates/edit_address.html")]
struct PersonAddressUpdateTemplate {
    should_warn: bool,
    candidate: Candidate,
    form: FormData<AddressForm>,
    full_list: FullCandidateList,
}

pub async fn edit_person_address(
    _: CandidateListEditAddressPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    Query(query): Query<InitialEditQuery>,
) -> AppResponse<impl IntoResponse> {
    let form = FormData::new_with_data(
        AddressForm::from(candidate.person.clone()),
        &context.csrf_tokens,
    );

    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            should_warn: query.should_warn(),
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
    full_list: FullCandidateList,
    candidate: Candidate,
    State(pool): State<PgPool>,
    Query(query): Query<InitialEditQuery>,
    Form(form): Form<AddressForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate.person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonAddressUpdateTemplate {
                should_warn: query.should_warn(),
                candidate,
                form: form_data,
                full_list,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::update_address(&pool, &person).await?;

            Ok(Redirect::to(&full_list.list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::Query,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        Context,
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

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&pool, list_id, person.id).await?;

        let response = edit_person_address(
            CandidateListEditAddressPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test(pool.clone()).await,
            full_list,
            candidate,
            Query(InitialEditQuery::new()),
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

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&pool, list_id, person.id).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.locality = "Rotterdam".to_string();

        let response = update_person_address(
            CandidateListEditAddressPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(pool.clone()),
            Query(InitialEditQuery::new()),
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

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&pool, list_id, person.id).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address(
            CandidateListEditAddressPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(pool.clone()),
            Query(InitialEditQuery::new()),
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
