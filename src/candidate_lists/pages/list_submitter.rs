use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context, HtmlTemplate,
    candidate_lists::{
        self, CandidateList, CandidateListSummary, ListSubmitterForm, pages::EditListSubmitterPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, Person},
    political_groups::{self, ListSubmitter},
};

#[derive(Template)]
#[template(path = "candidate_lists/list_submitter.html")]
struct ListSubmitterUpdateTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: i64,
    form: FormData<ListSubmitterForm>,
    candidate_list: CandidateList,
    list_submitters: Vec<ListSubmitter>,
}

pub async fn edit_list_submitter_form(
    _: EditListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_summary(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;
    let list_submitters =
        political_groups::get_list_submitters(&pool, context.political_group.id).await?;

    let form = FormData::new_with_data(
        ListSubmitterForm::from(candidate_list.clone()),
        &context.csrf_tokens,
    );

    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            candidate_lists,
            total_persons,
            form,
            candidate_list,
            list_submitters,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter(
    _: EditListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_summary(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;
    let list_submitters =
        political_groups::get_list_submitters(&pool, context.political_group.id).await?;
    match form.validate_update(&candidate_list, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterUpdateTemplate {
                candidate_lists,
                total_persons,
                form: form_data,
                candidate_list,
                list_submitters,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
            candidate_lists::update_candidate_list(&pool, &candidate_list).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}
