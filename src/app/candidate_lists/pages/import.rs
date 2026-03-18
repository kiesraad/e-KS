use askama::Template;
use axum::{
    Json,
    extract::Multipart,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    candidate_lists::{CandidateList, pages::CandidateListImportPath, structs::ListImportPayload},
    filters,
    form::FormData,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/import_export.html")]
struct ImportExportTemplate {
    list: CandidateList
}

pub async fn import_export(
    CandidateListImportPath { list_id}: CandidateListImportPath,
    context: Context,
    store: AppStore,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(ImportExportTemplate {
        list: store.get_candidate_list(list_id)?
    }, context).into_response())
}

pub async fn import_candidate_list(
    path: CandidateListImportPath,
    context: Context,
    store: AppStore,
    Json(payload): Json<ListImportPayload>,
) -> Result<Response, AppError> {
    todo!()
}
