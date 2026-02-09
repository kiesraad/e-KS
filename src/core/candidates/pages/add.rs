use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use serde::Deserialize;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    candidate_lists::{CandidateList, FullCandidateList},
    filters,
    persons::{Person, PersonId},
};

use super::AddCandidatePath;
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
    State(store): State<AppStore>,
) -> Result<impl IntoResponse, AppError> {
    let persons = full_list.list.persons_not_on_list(&store)?;

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
    mut list: CandidateList,
    State(store): State<AppStore>,
    Form(form): Form<AddPersonForm>,
) -> Result<Response, AppError> {
    let redirect = Redirect::to(&list.view_path()).into_response();
    let person_exists = store
        .get_persons()?
        .iter()
        .any(|person| person.id == form.person_id);

    if list.candidates.contains(&form.person_id) || !person_exists {
        return Ok(redirect);
    }

    list.append_candidate(&store, form.person_id).await?;

    Ok(redirect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;

    #[tokio::test]
    async fn view_candidate_list_renders_persons() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        person.create(&store).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = add_existing_person(
            AddCandidatePath { list_id },
            Context::new_test_without_db(),
            full_list,
            State(store),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.add_candidate_path()));
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_adds_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person.create(&store).await?;

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            list.clone(),
            State(store.clone()),
            Form(AddPersonForm {
                person_id: person.id,
            }),
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

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, person.id);

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_redirects_when_person_not_on_list() -> Result<(), AppError>
    {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let existing_person = sample_person_with_last_name(PersonId::new(), "Jansen");
        let new_person = sample_person_with_last_name(PersonId::new(), "Bakker");

        existing_person.create(&store).await?;
        list.candidates = vec![existing_person.id];
        list.create(&store).await?;
        new_person.create(&store).await?;

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            list.clone(),
            State(store.clone()),
            Form(AddPersonForm {
                person_id: new_person.id,
            }),
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

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, existing_person.id);
        assert_eq!(full_list.candidates[1].person.id, new_person.id);

        Ok(())
    }
}
