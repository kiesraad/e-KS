use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        pages::{CandidateListEditPersonPath, load_candidate_list},
        structs::{CandidateList, CandidateListEntry, FullCandidateList, MAX_CANDIDATES},
    },
    filters,
    form::{FormData, Validate},
    persons::{self, structs::PersonForm},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/update_person.html")]
struct PersonUpdateTemplate {
    full_list: FullCandidateList,
    candidate: CandidateListEntry,
    form: FormData<PersonForm>,
    max_candidates: usize,
}

pub(crate) async fn edit_person_form(
    CandidateListEditPersonPath {
        candidate_list,
        person,
    }: CandidateListEditPersonPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> AppResponse<impl IntoResponse> {
    let full_list = load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let candidate = full_list.get_candidate(&person, context.locale)?;

    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(PersonForm::from(candidate.person.clone()), &csrf_tokens),
            candidate,
            full_list,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

pub(crate) async fn update_person(
    CandidateListEditPersonPath {
        candidate_list,
        person,
    }: CandidateListEditPersonPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    let full_list = load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let candidate = full_list.get_candidate(&person, context.locale)?;

    match form.validate(Some(&candidate.person), app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                candidate,
                full_list,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::repository::update_person(&mut conn, &person).await?;

            // Redirect to the address edit page
            Ok(Redirect::to(&full_list.list.edit_person_path(&person.id)).into_response())
        }
    }
}
