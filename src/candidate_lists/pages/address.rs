use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        pages::{CandidateListEditAddressPath, load_candidate_list},
        structs::{CandidateList, CandidateListEntry, FullCandidateList, MAX_CANDIDATES},
    },
    filters,
    form::{FormData, Validate},
    persons::{self, structs::AddressForm},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/address.html")]
struct PersonAddressUpdateTemplate {
    candidate: CandidateListEntry,
    form: FormData<AddressForm>,
    full_list: FullCandidateList,
    max_candidates: usize,
}

pub(crate) async fn edit_person_address_form(
    CandidateListEditAddressPath {
        candidate_list,
        person,
    }: CandidateListEditAddressPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> AppResponse<impl IntoResponse> {
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let candidate = full_list.get_candidate(&person, context.locale)?;
    let form = FormData::new_with_data(AddressForm::from(candidate.person.clone()), &csrf_tokens);

    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            form,
            candidate: candidate.clone(),
            full_list,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

pub(crate) async fn update_person_address(
    CandidateListEditAddressPath {
        candidate_list,
        person,
    }: CandidateListEditAddressPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<AddressForm>,
) -> Result<Response, AppError> {
    let full_list = load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let candidate = full_list.get_candidate(&person, context.locale)?;

    match form.validate(Some(&candidate.person), app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonAddressUpdateTemplate {
                candidate,
                form: form_data,
                full_list,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::repository::update_person(&mut conn, &person).await?;

            Ok(Redirect::to(&full_list.list.view_path()).into_response())
        }
    }
}
