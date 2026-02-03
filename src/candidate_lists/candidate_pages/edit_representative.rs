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
        Candidate, CandidateList, FullCandidateList, candidate_pages::EditRepresentativePath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, InitialEditQuery, RepresentativeForm},
};

#[derive(Template)]
#[template(path = "candidates/edit_representative.html")]
struct EditRepresentativeTemplate {
    should_warn: bool,
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<RepresentativeForm>,
}

pub async fn edit_representative(
    _: EditRepresentativePath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    Query(query): Query<InitialEditQuery>,
) -> AppResponse<impl IntoResponse> {
    let form = FormData::new_with_data(
        RepresentativeForm::from(candidate.person.clone()),
        &context.csrf_tokens,
    );

    Ok(HtmlTemplate(
        EditRepresentativeTemplate {
            should_warn: query.should_warn(),
            candidate: candidate.clone(),
            full_list,
            form,
        },
        context,
    ))
}

pub async fn update_representative(
    _: EditRepresentativePath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    State(pool): State<PgPool>,
    Query(query): Query<InitialEditQuery>,
    Form(form): Form<RepresentativeForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate.person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            EditRepresentativeTemplate {
                should_warn: query.should_warn(),
                candidate,
                full_list,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::update_representative(&pool, &person).await?;

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
            extract_csrf_token, response_body_string, sample_candidate_list, sample_person,
            sample_representative_form,
        },
    };

    #[sqlx::test]
    async fn edit_representative_renders_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
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

        let response = edit_representative(
            EditRepresentativePath {
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
    async fn edit_representative_renders_valid_csrf_token(pool: PgPool) -> Result<(), sqlx::Error> {
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
        let csrf_tokens = context.csrf_tokens.clone();

        let response = edit_representative(
            EditRepresentativePath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            Query(InitialEditQuery::new()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        let csrf_token = extract_csrf_token(&body).expect("csrf token");
        assert!(csrf_tokens.consume(&csrf_token));

        Ok(())
    }

    #[sqlx::test]
    async fn update_representative_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
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
        let mut form = sample_representative_form(&csrf_token);
        form.representative_last_name = "Smit".to_string();

        let response = update_representative(
            EditRepresentativePath {
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
        assert_eq!(updated.representative_last_name, Some("Smit".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_representative_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
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
        let mut form = sample_representative_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_representative(
            EditRepresentativePath {
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
        assert!(body.contains("The postal code is not valid"));

        Ok(())
    }
}
