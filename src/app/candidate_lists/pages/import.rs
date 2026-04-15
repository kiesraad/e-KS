use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, AppEvent, Context, HtmlTemplate, RequestCtx,
    candidate_lists::{
        CandidateList,
        importer::{ImportCandidateListError, import_candidate_list_csv},
        pages::CandidateListImportPath,
    },
    filters,
    form::{EmptyForm, FileForm, FormData},
    redirect_success, trans,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/import_export.html")]
struct ImportExportTemplate {
    list: CandidateList,
    import_errors: Vec<String>,
    form: FormData<EmptyForm>,
}

fn render_import_export(
    list: CandidateList,
    import_errors: Vec<String>,
    context: Context,
) -> Response {
    HtmlTemplate(
        ImportExportTemplate {
            list,
            import_errors,
            form: FormData::new(&context.session.csrf_tokens),
        },
        context,
    )
    .into_response()
}

pub async fn import_export(
    CandidateListImportPath { list_id }: CandidateListImportPath,
    ctx: RequestCtx,
) -> Result<Response, AppError> {
    Ok(render_import_export(
        ctx.store.get_candidate_list(list_id)?,
        vec![],
        ctx.context,
    ))
}

pub async fn import_candidate_list(
    CandidateListImportPath { list_id }: CandidateListImportPath,
    ctx: RequestCtx,
    import_data: FileForm,
) -> Result<Response, AppError> {
    let RequestCtx { context, store, .. } = ctx;
    let mut list = store.get_candidate_list(list_id)?;
    let csrf_form = EmptyForm {
        csrf_token: import_data.csrf_token,
    };

    if csrf_form
        .validate_create(&context.session.csrf_tokens)
        .is_err()
    {
        return Err(AppError::CsrfTokenInvalid);
    }

    let Some(csv_data) = import_data.file_data else {
        return Ok(render_import_export(
            list.clone(),
            vec![trans!(
                "candidate_list.import_errors.missing_file",
                context.session.locale
            )],
            context,
        ));
    };

    let file_size = csv_data.len();

    match import_candidate_list_csv(
        &mut list,
        &store,
        &csv_data,
        &context.session.csrf_tokens,
        context.session.locale,
    )
    .await
    {
        Ok(()) => {
            store
                .update(AppEvent::ImportCsv {
                    file_name: import_data.file_name.unwrap_or_default(),
                    file_size,
                    list_id,
                })
                .await?;
            Ok(redirect_success(list.view_path()))
        }
        Err(ImportCandidateListError::App(error)) => Err(error),
        Err(ImportCandidateListError::Messages(messages)) => {
            Ok(render_import_export(list.clone(), messages, context))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Bytes, http::StatusCode};

    use crate::{
        AppStore, Context, QueryParamState, RequestCtx,
        candidate_lists::CandidateListId,
        test_utils::{response_body_string, sample_candidate_list},
    };

    #[tokio::test]
    async fn import_export_renders_multipart_file_form() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let response = import_export(
            CandidateListImportPath { list_id: list.id },
            RequestCtx {
                context: Context::new_test_without_db(),
                store,
                query: QueryParamState::default(),
            },
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("type=\"file\""));
        assert!(body.contains("name=\"file_data\""));
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("data-import-file-field"));
        assert!(body.contains("data-import-file-trigger"));
        assert!(body.contains("formenctype=\"multipart/form-data\""));
        assert!(!body.contains("one-click-upload"));

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_invalid_csv_renders_error() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            RequestCtx {
                context,
                store: store.clone(),
                query: QueryParamState::default(),
            },
            FileForm {
                csrf_token,
                file_name: Some("invalid.csv".to_string()),
                file_data: Some(Bytes::from(invalid_csv())),
            },
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert!(body.contains("The candidate on line 1 could not be imported:"));
        assert!(body.contains("has 2 columns, but earlier rows have 23"));
        assert_eq!(body.matches("alert alert-warning").count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_renders_multiple_validation_errors() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            RequestCtx {
                context,
                store,
                query: QueryParamState::default(),
            },
            FileForm {
                csrf_token,
                file_name: Some("validation-errors.csv".to_string()),
                file_data: Some(Bytes::from(csv_with_multiple_validation_errors())),
            },
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert_eq!(
            body.matches("The candidate on line 1 could not be imported.")
                .count(),
            2
        );
        assert!(body.contains("Initials"));
        assert!(body.contains("The provided value is not valid."));
        assert!(body.contains("Postal code"));
        assert!(body.contains("The postal code is not valid, use the format 1234AB."));
        assert_eq!(body.matches("alert alert-warning").count(), 2);

        Ok(())
    }

    const CSV_HEADER: &str = include_str!("../testdata/csv_header.csv");

    fn csv_headers() -> &'static str {
        CSV_HEADER.trim_end_matches('\n').trim_end_matches('\r')
    }

    fn invalid_csv() -> String {
        format!("{}\r\n{row}", csv_headers(), row = "H.A.H.A.,Henk\r\n")
    }

    fn csv_with_multiple_validation_errors() -> String {
        format!(
            "{}\r\n{}",
            csv_headers(),
            "JD,Henk,,Jansen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1000,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
        )
    }
}
