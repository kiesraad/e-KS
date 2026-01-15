use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    candidate_lists::{
        self,
        structs::{CandidateList, FullCandidateList, MAX_CANDIDATES},
    },
    filters,
    persons::{self, structs::Person},
    t,
};

use super::{CandidateListAddPersonPath, load_candidate_list};

#[derive(Template)]
#[template(path = "candidate_lists/add_existing_person.html")]
struct AddExistingPersonTemplate {
    full_list: FullCandidateList,
    persons: Vec<Person>,
    max_candidates: usize,
}

pub async fn add_existing_person(
    CandidateListAddPersonPath { id }: CandidateListAddPersonPath,
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
    CandidateListAddPersonPath { id }: CandidateListAddPersonPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<AddPersonForm>,
) -> Result<Response, AppError> {
    let full_list = load_candidate_list(&mut conn, &id, context.locale).await?;

    if full_list.get_index(&form.person_id).is_some() {
        return Ok(Redirect::to(&full_list.list.view_path()).into_response());
    }

    let mut person_ids = full_list.get_ids();
    person_ids.push(form.person_id);
    candidate_lists::repository::update_candidate_list_order(&mut conn, &id, &person_ids).await?;

    Ok(Redirect::to(&full_list.list.edit_person_address_path(&form.person_id)).into_response())
}
