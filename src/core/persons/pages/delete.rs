use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context,
    candidate_lists::CandidateList,
    form::EmptyForm,
    persons::{Person, pages::DeletePersonPath},
};

pub async fn delete_person(
    DeletePersonPath { person_id }: DeletePersonPath,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            // TODO: set error flash message
            Ok(Redirect::to(&Person::list_path()).into_response())
        }
        Ok(_) => {
            CandidateList::remove_candidate_from_all(&store, person_id).await?;

            Person::delete_by_id(&store, person_id).await?;
            // TODO: set success flash message
            Ok(Redirect::to(&Person::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppError, AppStore, Context, persons::PersonId, test_utils::sample_person};
    use axum_extra::extract::Form;

    #[tokio::test]
    async fn delete_person_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_person(
            DeletePersonPath { person_id },
            context,
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
        assert_eq!(location, Person::list_path());

        let found = store.get_person(person_id);
        assert!(found.is_err());

        Ok(())
    }
}
