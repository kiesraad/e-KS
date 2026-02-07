use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppStore, Context, HtmlTemplate, filters,
    form::FormData,
    persons::{InitialQuery, Person, RepresentativeForm, pages::UpdateRepresentativePath},
};

#[derive(Template)]
#[template(path = "persons/update_representative.html")]
struct RepresentativeUpdateTemplate {
    should_warn: bool,
    person: Person,
    form: FormData<RepresentativeForm>,
}

pub async fn update_representative(
    _: UpdateRepresentativePath,
    context: Context,
    person: Person,
    Query(query): Query<InitialQuery>,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        RepresentativeUpdateTemplate {
            should_warn: query.should_warn(),
            form: FormData::new_with_data(
                RepresentativeForm::from(person.clone()),
                &context.csrf_tokens,
            ),
            person,
        },
        context,
    ))
}

pub async fn update_representative_submit(
    _: UpdateRepresentativePath,
    context: Context,
    person: Person,
    State(store): State<AppStore>,
    Query(query): Query<InitialQuery>,
    Form(form): Form<RepresentativeForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            RepresentativeUpdateTemplate {
                should_warn: query.should_warn(),
                person,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            person.update(&store).await?;

            Ok(Redirect::to(&Person::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context,
        persons::PersonId,
        test_utils::{
            extract_csrf_token, response_body_string, sample_person, sample_representative_form,
        },
    };
    use axum::{
        extract::Query,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;

    #[tokio::test]
    async fn update_representative_renders_existing_person() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let response = update_representative(
            UpdateRepresentativePath { person_id },
            Context::new_test_without_db(),
            person,
            Query(InitialQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn update_representative_renders_valid_csrf_token() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_tokens = context.csrf_tokens.clone();

        let response = update_representative(
            UpdateRepresentativePath { person_id },
            context,
            person,
            Query(InitialQuery::default()),
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

    #[tokio::test]
    async fn update_representative_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_representative_form(&csrf_token);
        form.representative.last_name = "Smit".to_string();

        let response = update_representative_submit(
            UpdateRepresentativePath { person_id },
            context,
            person,
            State(store.clone()),
            Query(InitialQuery::default()),
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

        let updated = store.get_person(person_id)?;
        assert_eq!(updated.representative.last_name.to_string(), "Smit");

        Ok(())
    }

    #[tokio::test]
    async fn update_representative_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_representative_form(&csrf_token);
        form.address.postal_code = "a".to_string();

        let response = update_representative_submit(
            UpdateRepresentativePath { person_id },
            context,
            person,
            State(store),
            Query(InitialQuery::default()),
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
