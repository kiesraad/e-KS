use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use chrono::Utc;

use crate::{
    AppError, AppEvent, AppResponse, AppStore, Context, HtmlTemplate,
    candidate_lists::FullCandidateList,
    candidates::Candidate,
    filters,
    form::{FormData, Validate},
    persons::{AddressForm, InitialEditQuery},
};

use super::CandidateListUpdateAddressPath;
#[derive(Template)]
#[template(path = "candidates/update_address.html")]
struct PersonAddressUpdateTemplate {
    should_warn: bool,
    candidate: Candidate,
    form: FormData<AddressForm>,
    full_list: FullCandidateList,
}

pub async fn update_person_address(
    _: CandidateListUpdateAddressPath,
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

pub async fn update_person_address_submit(
    _: CandidateListUpdateAddressPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    State(store): State<AppStore>,
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
        Ok(mut person) => {
            person.updated_at = Utc::now();
            store.update(AppEvent::UpdatePerson(person)).await?;

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
        AppEvent, AppStore, Context,
        candidate_lists::{CandidateList, CandidateListId},
        persons::PersonId,
        test_utils::{
            response_body_string, sample_address_form, sample_candidate_list,
            sample_person_with_last_name,
        },
    };

    #[sqlx::test]
    async fn update_person_address_renders_candidate(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let response = update_person_address(
            CandidateListUpdateAddressPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test_without_db(),
            full_list,
            candidate,
            Query(InitialEditQuery::default()),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.locality = "Rotterdam".to_string();

        let response = update_person_address_submit(
            CandidateListUpdateAddressPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(store.clone()),
            Query(InitialEditQuery::default()),
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
        assert_eq!(location, list.view_path());

        let updated = store
            .get_persons()?
            .into_iter()
            .find(|p| p.id == person.id)
            .expect("updated person");
        assert_eq!(updated.locality, Some("Rotterdam".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address_submit(
            CandidateListUpdateAddressPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(store),
            Query(InitialEditQuery::default()),
            Form(form),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The postal code is not valid"));

        Ok(())
    }
}
