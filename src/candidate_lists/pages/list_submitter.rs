use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context, HtmlTemplate,
    candidate_lists::{
        self, CandidateList, CandidateListSummary, ListSubmitterForm, pages::AddListSubmitterPath,
    },
    filters,
    form::FormData,
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

pub async fn update_list_submitter_form(
    _: AddListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_summary(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;
    let list_submitters =
        political_groups::get_list_submitters(&pool, context.political_group.id).await?;

    let form = FormData::new_with_data(
        ListSubmitterForm {
            list_submitter_id: form.list_submitter_id,
            csrf_token: context.csrf_tokens.issue().value,
        },
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
