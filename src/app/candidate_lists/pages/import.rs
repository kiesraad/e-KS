use askama::Template;
use axum::{
    body::Bytes,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    candidate_lists::{CandidateList, pages::CandidateListImportPath, structs::CandidateRecord},
    core::{Csv, CsvError},
    filters,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/import_export.html")]
struct ImportExportTemplate {
    list: CandidateList,
    import_errors: Vec<CsvError>,
}

pub async fn import_export(
    CandidateListImportPath { list_id }: CandidateListImportPath,
    context: Context,
    store: AppStore,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        ImportExportTemplate {
            list: store.get_candidate_list(list_id)?,
            import_errors: vec![],
        },
        context,
    )
    .into_response())
}

pub async fn import_candidate_list(
    CandidateListImportPath { list_id }: CandidateListImportPath,
    context: Context,
    store: AppStore,
    csv_data: Bytes,
) -> Result<Response, AppError> {
    let records = match Csv::<CandidateRecord>::from_bytes(&csv_data) {
        Ok(records) => records,
        Err(errors) => {
            return Ok(HtmlTemplate(ImportExportTemplate {
                list: store.get_candidate_list(list_id)?,
                import_errors: errors
            }, context).into_response());
        }
    };
    todo!()
}
