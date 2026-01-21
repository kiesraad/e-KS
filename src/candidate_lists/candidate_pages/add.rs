use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;
use serde::Deserialize;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    candidate_lists::{self, CandidateList, FullCandidateList, pages::AddCandidatePath},
    filters,
    persons::{self, Person, PersonId},
    t,
};

#[derive(Template)]
#[template(path = "candidates/add_existing.html")]
struct AddExistingPersonTemplate {
    full_list: FullCandidateList,
    persons: Vec<Person>,
}

pub async fn add_existing_person(
    _: AddCandidatePath,
    context: Context,
    full_list: FullCandidateList,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let persons = persons::list_persons_not_on_candidate_list(&mut conn, full_list.id()).await?;

    Ok(HtmlTemplate(
        AddExistingPersonTemplate { full_list, persons },
        context,
    ))
}

#[derive(Deserialize)]
pub struct AddPersonForm {
    pub person_id: PersonId,
}

pub async fn add_person_to_candidate_list(
    _: AddCandidatePath,
    full_list: FullCandidateList,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<AddPersonForm>,
) -> Result<Response, AppError> {
    let redirect = Redirect::to(&full_list.list.view_path()).into_response();
    let person = persons::get_person(&mut conn, form.person_id).await?;

    if full_list.contains(form.person_id) || person.is_none() {
        return Ok(redirect);
    }

    candidate_lists::append_candidate_to_list(&mut conn, full_list.id(), form.person_id).await?;

    Ok(redirect)
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
        persons::{self, PersonId},
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };

    #[sqlx::test]
    async fn view_candidate_list_renders_persons(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");

        let response = add_existing_person(
            AddCandidatePath { list_id },
            Context::new_test(),
            full_list,
            DbConnection(pool.acquire().await?),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.add_candidate_path()));
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn add_person_to_candidate_list_adds_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            DbConnection(pool.acquire().await?),
            Form(AddPersonForm {
                person_id: person.id,
            }),
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
        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, person.id);

        Ok(())
    }

    #[sqlx::test]
    async fn add_person_to_candidate_list_redirects_when_person_not_on_list(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let existing_person = sample_person_with_last_name(PersonId::new(), "Jansen");
        let new_person = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &existing_person).await?;
        persons::create_person(&mut conn, &new_person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[existing_person.id])
            .await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            DbConnection(pool.acquire().await?),
            Form(AddPersonForm {
                person_id: new_person.id,
            }),
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
        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, existing_person.id);
        assert_eq!(full_list.candidates[1].person.id, new_person.id);

        Ok(())
    }
}
