use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, AppResponse, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{
        self, InitialEditQuery, Person, PersonPagination, PersonSort, RepresentativeForm,
        pages::EditRepresentativePath,
    },
};

#[derive(Template)]
#[template(path = "persons/representative.html")]
struct RepresentativeUpdateTemplate {
    should_warn: bool,
    person: Person,
    form: FormData<RepresentativeForm>,
    person_pagination: PersonPagination,
}

pub async fn edit_representative(
    _: EditRepresentativePath,
    context: Context,
    person: Person,
    person_pagination: PersonPagination,
    Query(query): Query<InitialEditQuery>,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        RepresentativeUpdateTemplate {
            should_warn: query.should_warn(),
            form: FormData::new_with_data(
                RepresentativeForm::from(person.clone()),
                &context.csrf_tokens,
            ),
            person,
            person_pagination,
        },
        context,
    ))
}

pub async fn update_representative(
    _: EditRepresentativePath,
    context: Context,
    person: Person,
    State(pool): State<PgPool>,
    person_pagination: PersonPagination,
    Query(query): Query<InitialEditQuery>,
    Form(form): Form<RepresentativeForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            RepresentativeUpdateTemplate {
                should_warn: query.should_warn(),
                person,
                form: form_data,
                person_pagination,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::update_representative(&pool, &person).await?;

            Ok(Redirect::to(&Person::list_path()).into_response())
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
        persons::{self, PersonId},
        test_utils::{response_body_string, sample_person, sample_representative_form},
    };

    #[sqlx::test]
    async fn edit_representative_renders_existing_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        persons::create_person(&pool, &person).await?;

        let response = edit_representative(
            EditRepresentativePath { person_id },
            Context::new_test(pool.clone()).await,
            person,
            PersonPagination::empty(),
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
    async fn update_representative_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        persons::create_person(&pool, &person).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_representative_form(&csrf_token);
        form.representative_last_name = "Smit".to_string();

        let response = update_representative(
            EditRepresentativePath { person_id },
            context,
            person,
            State(pool.clone()),
            PersonPagination::empty(),
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
        assert_eq!(location, Person::list_path());

        let updated = persons::get_person(&pool, person_id)
            .await?
            .expect("updated person");
        assert_eq!(updated.representative_last_name, Some("Smit".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_representative_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        persons::create_person(&pool, &person).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_representative_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_representative(
            EditRepresentativePath { person_id },
            context,
            person,
            State(pool.clone()),
            PersonPagination::empty(),
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
