use axum::response::Response;

use crate::{
    AppError, Form, RequestCtx,
    form::EmptyForm,
    persons::{Person, pages::DeletePersonPath},
    redirect_success,
};

pub async fn delete_person(
    _: DeletePersonPath,
    ctx: RequestCtx,
    person: Person,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(ctx.csrf()) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            person.delete(&ctx.store).await?;

            Ok(redirect_success(Person::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState, RequestCtx, persons::PersonId,
        test_utils::sample_person,
    };

    #[tokio::test]
    async fn delete_person_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;

        let response = delete_person(
            DeletePersonPath { person_id },
            RequestCtx {
                context,
                store: store.clone(),
                query: QueryParamState::default(),
            },
            person,
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
