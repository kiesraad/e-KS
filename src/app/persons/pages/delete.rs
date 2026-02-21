use axum::{extract::State, response::Response};

use crate::{
    AppError, Context, Form, Store,
    form::EmptyForm,
    persons::{Person, pages::DeletePersonPath},
    redirect_success,
};

pub async fn delete_person(
    _: DeletePersonPath,
    context: Context,
    person: Person,
    State(store): State<Store>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            person.delete(&store).await?;

            Ok(redirect_success(Person::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::{
        AppError, Context, Form, QueryParamState, Store, persons::PersonId,
        test_utils::sample_person,
    };

    #[tokio::test]
    async fn delete_person_removes_and_redirects() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_person(
            DeletePersonPath { person_id },
            context,
            person,
            State(store.clone()),
            Form(EmptyForm::new(csrf_token)),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(
            location,
            Person::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let found = store.get_person(person_id);
        assert!(found.is_err());

        Ok(())
    }
}
