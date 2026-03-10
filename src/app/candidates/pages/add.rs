use askama::Template;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate,
    candidate_lists::{CandidateListId, FullCandidateList},
    candidates::{AddPerson, AddPersonAction, AddPersonForm},
    filters,
    form::FormData,
    persons::{Person, PersonId},
};

use super::AddCandidatePath;

#[derive(Template)]
#[template(path = "candidates/pages/add.html")]
struct AddExistingPersonTemplate {
    full_list: FullCandidateList,
    persons: Vec<Person>,
    added_candidates: HashMap<PersonId, usize>,
    form: FormData<AddPersonForm>,
    show_add_all: bool,
}

impl AddExistingPersonTemplate {
    /// Creates a template for adding an existing person to the candidate list.
    /// If `added_position` is provided, the template will show all candidates from that position onward as already added to the list.
    fn from(
        list_id: CandidateListId,
        added_position: Option<usize>,
        store: &AppStore,
        form: FormData<AddPersonForm>,
    ) -> Result<Self, AppError> {
        let full_list = FullCandidateList::get(store, list_id)?;
        let added_candidates = match added_position {
            Some(pos) => full_list
                .candidates
                .iter()
                .filter(|candidate| candidate.position >= pos)
                .map(|candidate| (candidate.person.id, candidate.position))
                .collect::<HashMap<PersonId, usize>>(),
            None => HashMap::new(),
        };
        let candidate_ids = added_candidates.keys().cloned().collect::<Vec<_>>();
        let persons = full_list.list.persons_not_on_list(store, &candidate_ids)?;

        Ok(Self {
            show_add_all: persons.len() != candidate_ids.len(),
            full_list,
            persons,
            added_candidates,
            form,
        })
    }
}

/// Handles the logic for adding a person to the candidate list based on the submitted form data.
async fn handle_add_candidate_form(
    add_person: &mut AddPerson,
    full_list: &mut FullCandidateList,
    store: &AppStore,
) -> Result<(), AppError> {
    match add_person.action {
        AddPersonAction::None => {
            // No action, do nothing.
        }
        AddPersonAction::AddAll => {
            // Enable showing the newly added candidate as already added to the list in the template.
            if add_person.added_position.is_none() {
                add_person.added_position = Some(full_list.list.candidates.len() + 1);
            }

            let persons_not_on_list = full_list.list.persons_not_on_list(store, &[])?;
            let person_ids = persons_not_on_list
                .iter()
                .map(|person| person.id)
                .collect::<Vec<_>>();
            let mut all_persons = full_list.list.candidates.clone();
            all_persons.extend(person_ids);

            full_list.list.update_order(store, &all_persons).await?;
        }
        AddPersonAction::TogglePerson(person_id) => {
            if full_list.list.candidates.contains(&person_id) {
                full_list.list.remove_candidate(store, person_id).await?;
            } else {
                full_list.list.append_candidate(store, person_id).await?;

                // Enable showing the newly added candidate as already added to the list in the template.
                if add_person.added_position.is_none() {
                    add_person.added_position = Some(full_list.list.candidates.len());
                }
            }
        }
    }

    Ok(())
}

pub async fn add_existing_person(
    AddCandidatePath { list_id }: AddCandidatePath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        AddExistingPersonTemplate::from(
            list_id,
            None,
            &store,
            FormData::new(&context.session.csrf_tokens),
        )?,
        context,
    ))
}

pub async fn add_person_to_candidate_list(
    _: AddCandidatePath,
    mut full_list: FullCandidateList,
    store: AppStore,
    mut context: Context,
    Form(form): Form<AddPersonForm>,
) -> Result<Response, AppError> {
    context.show_success_alert = true;

    match form.validate_create(&context.session.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            AddExistingPersonTemplate::from(
                full_list.list.id,
                form_data.data.added_position.parse().ok(),
                &store,
                form_data,
            )?,
            context,
        )
        .into_response()),
        Ok(mut add_person) => {
            handle_add_candidate_form(&mut add_person, &mut full_list, &store).await?;

            Ok(HtmlTemplate(
                AddExistingPersonTemplate::from(
                    full_list.list.id,
                    add_person.added_position,
                    &store,
                    FormData::new_with_data(add_person.into(), &context.session.csrf_tokens),
                )?,
                context,
            )
            .into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context, Form, TokenValue,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn view_candidate_list_renders_persons() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        person.create(&store).await?;

        let response = add_existing_person(
            AddCandidatePath { list_id },
            Context::new_test_without_db(),
            store,
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_adds_and_renders() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = AddPersonForm {
            action: person.id.to_string(),
            added_position: String::new(),
            csrf_token,
        };

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            store.clone(),
            context,
            Form(form),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, person.id);

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_adds_when_person_not_on_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let existing_person = sample_person_with_last_name(PersonId::new(), "Jansen");
        let new_person = sample_person_with_last_name(PersonId::new(), "Bakker");

        existing_person.create(&store).await?;
        list.candidates = vec![existing_person.id];
        list.create(&store).await?;
        new_person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = AddPersonForm {
            action: new_person.id.to_string(),
            added_position: String::new(),
            csrf_token,
        };

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            store.clone(),
            context,
            Form(form),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, existing_person.id);
        assert_eq!(full_list.candidates[1].person.id, new_person.id);

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_add_all_adds_missing_persons() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let existing_person = sample_person_with_last_name(PersonId::new(), "Adams");
        let person_one = sample_person_with_last_name(PersonId::new(), "Bakker");
        let person_two = sample_person_with_last_name(PersonId::new(), "Jansen");

        existing_person.create(&store).await?;
        person_one.create(&store).await?;
        person_two.create(&store).await?;
        list.candidates = vec![existing_person.id];
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = AddPersonForm {
            action: AddPersonAction::AddAll.to_string(),
            added_position: String::new(),
            csrf_token,
        };

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            store.clone(),
            context,
            Form(form),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 3);
        assert!(full_list.contains(existing_person.id));
        assert!(full_list.contains(person_one.id));
        assert!(full_list.contains(person_two.id));

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_invalid_csrf_does_not_add() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let form = AddPersonForm {
            action: person.id.to_string(),
            added_position: String::new(),
            csrf_token: TokenValue("invalid".to_string()),
        };

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            store.clone(),
            context,
            Form(form),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert!(full_list.candidates.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_removes_candidate() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let remove_person = sample_person_with_last_name(PersonId::new(), "Jansen");
        let keep_person = sample_person_with_last_name(PersonId::new(), "Bakker");

        remove_person.create(&store).await?;
        keep_person.create(&store).await?;
        list.candidates = vec![remove_person.id, keep_person.id];
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = AddPersonForm {
            action: remove_person.id.to_string(),
            added_position: String::new(),
            csrf_token,
        };

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            store.clone(),
            context,
            Form(form),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, keep_person.id);

        Ok(())
    }
}
