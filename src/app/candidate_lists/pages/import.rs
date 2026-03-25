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
    import_errors: Vec<String>,
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
            return Ok(HtmlTemplate(
                ImportExportTemplate {
                    list: store.get_candidate_list(list_id)?,
                    import_errors: errors
                        .into_iter()
                        .map(|error| error.message(context.session.locale))
                        .collect(),
                },
                context,
            )
            .into_response());
        }
    };

    let mut persons = Vec::new();

    for (index, record) in records.into_iter().enumerate() {
        let person = match record.validate_create(&context.session.csrf_tokens) {
            Ok(person) => person,
            Err(error) => {
                return Ok(HtmlTemplate(
                    ImportExportTemplate {
                        list: store.get_candidate_list(list_id)?,
                        import_errors: error
                            .errors()
                            .into_iter()
                            .map(|(field_name, error)| {
                                CsvError::ParseError {
                                    candidate_number: index + 1,
                                    field_name,
                                    message: error.message(context.session.locale),
                                }
                                .message(context.session.locale)
                            })
                            .collect(),
                    },
                    context,
                )
                .into_response());
            }
        };

        persons.push(person);
    }

    dbg!(
        "imported {} candidates successfully: {:?}",
        persons.len(),
        persons
    );
    todo!("actually import the candidates into the list");
}
