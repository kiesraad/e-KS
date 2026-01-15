use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    candidate_lists::{
        self, CandidateList, FullCandidateList, MAX_CANDIDATES,
        pages::{AddCandidatePath, load_candidate_list},
    },
    filters,
    persons::{self, Person},
    t,
};

#[derive(Template)]
#[template(path = "candidates/add_existing.html")]
struct AddExistingPersonTemplate {
    full_list: FullCandidateList,
    persons: Vec<Person>,
    max_candidates: usize,
}

pub async fn add_existing_person(
    AddCandidatePath { id }: AddCandidatePath,
    context: Context,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let full_list: FullCandidateList = load_candidate_list(&mut conn, &id, context.locale).await?;
    let persons = persons::repository::list_persons_not_on_candidate_list(&mut conn, &id).await?;

    Ok(HtmlTemplate(
        AddExistingPersonTemplate {
            full_list,
            persons,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

#[derive(Deserialize)]
pub(crate) struct AddPersonForm {
    pub person_id: Uuid,
}

pub(crate) async fn add_person_to_candidate_list(
    AddCandidatePath { id }: AddCandidatePath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<AddPersonForm>,
) -> Result<Response, AppError> {
    let full_list = load_candidate_list(&mut conn, &id, context.locale).await?;
    let redirect = Redirect::to(&full_list.list.view_path()).into_response();

    if full_list.get_index(&form.person_id).is_some() {
        return Ok(redirect);
    }

    let mut person_ids = full_list.get_ids();
    person_ids.push(form.person_id);
    candidate_lists::repository::update_candidate_list_order(&mut conn, &id, &person_ids).await?;

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
    use uuid::Uuid;

    use crate::{
        Context, DbConnection, Locale, candidate_lists, persons,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };

    #[sqlx::test]
    async fn view_candidate_list_renders_persons(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person(Uuid::new_v4());

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let response = add_existing_person(
            AddCandidatePath { id: list_id },
            Context::new(Locale::En),
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
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(Uuid::new_v4(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let response = add_person_to_candidate_list(
            AddCandidatePath { id: list_id },
            Context::new(Locale::En),
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
        let full_list = load_candidate_list(&mut conn, &list_id, Locale::En)
            .await
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, person.id);

        Ok(())
    }
}
