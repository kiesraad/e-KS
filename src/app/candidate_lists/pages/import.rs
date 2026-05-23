use askama::Template;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, HtmlTemplate, Locale,
    candidate_lists::{
        CandidateList,
        importer::{ImportCandidateListError, import_candidate_list_csv},
        pages::{CandidateListImportPath, CandidateListImportTemplatePath},
        structs::{CSV_HEADERS, CandidateRecordCsv},
    },
    core::Csv,
    filters,
    form::{EmptyForm, FileForm, FormData},
    redirect_success, trans,
};

pub(crate) const MAX_IMPORT_SIZE_BYTES: usize = 5 * 1024 * 1024;
const MAX_IMPORT_SIZE_MB: usize = MAX_IMPORT_SIZE_BYTES / (1024 * 1024);

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
            form: FormData::new(&context.session.csrf_token),
        },
        context,
    )
    .into_response()
}

pub async fn import_export(
    CandidateListImportPath { list_id }: CandidateListImportPath,
    context: Context,
    store: AppStore,
) -> Result<Response, AppError> {
    Ok(render_import_export(
        store.get_candidate_list(list_id)?,
        vec![],
        context,
    ))
}

pub async fn import_candidate_list(
    CandidateListImportPath { list_id }: CandidateListImportPath,
    context: Context,
    store: AppStore,
    import_data: Result<FileForm, AppError>,
) -> Result<Response, AppError> {
    let mut list = store.get_candidate_list(list_id)?;

    let import_data = match import_data {
        Ok(form) => form,
        Err(err) => match upload_error_messages(&err, context.session.locale) {
            Some(messages) => return Ok(render_import_export(list, messages, context)),
            None => return Err(err),
        },
    };

    let csrf_form = EmptyForm {
        csrf_token: import_data.csrf_token,
    };

    if csrf_form
        .validate_create(&context.session.csrf_token)
        .is_err()
    {
        return Err(AppError::CsrfTokenInvalid);
    }

    if let Some(name) = &import_data.file_name
        && !has_csv_extension(name)
    {
        return Ok(render_import_export(
            list,
            vec![trans!(
                "candidate_list.import_errors.invalid_file_type",
                context.session.locale
            )],
            context,
        ));
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
        &context.session.csrf_token,
        context.session.locale,
        import_data.file_name.unwrap_or_default(),
        file_size,
    )
    .await
    {
        Ok(()) => Ok(redirect_success(list.view_path())),
        Err(ImportCandidateListError::App(error)) => Err(error),
        Err(ImportCandidateListError::Messages(messages)) => {
            Ok(render_import_export(list.clone(), messages, context))
        }
    }
}

/// The upload field's HTML `accept=".csv,text/csv"` hint is browser-side only.
/// Reject any uploaded file whose name does not end in `.csv` (case-insensitive)
/// before handing it to the CSV parser.
fn has_csv_extension(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered.ends_with(".csv") && lowered.len() > ".csv".len()
}

/// Translate a `FileForm` extraction failure into inline error messages, or
/// `None` for failures that should keep falling through to the global error
/// page (e.g. unrelated `AppError` variants).
fn upload_error_messages(err: &AppError, locale: Locale) -> Option<Vec<String>> {
    match err {
        AppError::MultipartFormError(e) if e.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            Some(vec![trans!(
                "candidate_list.import_errors.file_too_large",
                locale,
                MAX_IMPORT_SIZE_MB
            )])
        }
        AppError::MultipartFormError(_) | AppError::MultipartError(_) => Some(vec![trans!(
            "candidate_list.import_errors.upload_failed",
            locale
        )]),
        _ => None,
    }
}

pub async fn download_import_template(
    _: CandidateListImportTemplatePath,
) -> Result<Response, AppError> {
    let file_name = "kandidatenlijst-export-sjabloon.csv";
    let (response, file_size) = Csv::<CandidateRecordCsv> {
        filename: file_name.to_string(),
        headers: Some(CSV_HEADERS.to_vec()),
        records: vec![],
    }
    .generate_csv_response()?;

    tracing::info!(
        file_name,
        content_type = "text/csv",
        size_bytes = file_size,
        "file download served",
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::{Body, Bytes},
        extract::DefaultBodyLimit,
        http::{Request, StatusCode},
    };
    use axum_extra::routing::RouterExt;
    use tower::ServiceExt;

    use crate::{
        AppState, Locale, Session, StreamId,
        candidate_lists::CandidateListId,
        test_utils::{response_body_string, sample_candidate_list},
    };

    const TEST_BOUNDARY: &str = "----eks-test-boundary";

    fn multipart_body(csrf_token: &str, csv: &str) -> String {
        format!(
            "--{TEST_BOUNDARY}\r\n\
             Content-Disposition: form-data; name=\"csrf_token\"\r\n\r\n\
             {csrf_token}\r\n\
             --{TEST_BOUNDARY}\r\n\
             Content-Disposition: form-data; name=\"file_data\"; filename=\"candidates.csv\"\r\n\
             Content-Type: text/csv\r\n\r\n\
             {csv}\r\n\
             --{TEST_BOUNDARY}--\r\n"
        )
    }

    async fn import_via_router(
        store: AppStore,
        list_id: CandidateListId,
        body: Body,
        body_limit: usize,
    ) -> axum::response::Response {
        let app_state = AppState::new_for_tests().await;
        let app: Router = Router::new()
            .typed_post(import_candidate_list)
            .layer(DefaultBodyLimit::max(body_limit))
            .with_state(app_state);

        let mut request = Request::builder()
            .method("POST")
            .uri(format!("/candidate-lists/{list_id}/import"))
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={TEST_BOUNDARY}"),
            )
            .body(body)
            .unwrap();

        let mut session = Session::new_test_with_locale(Locale::En);
        session.set_stream_id(StreamId::new());
        request.extensions_mut().insert(session);
        request.extensions_mut().insert(store);

        app.oneshot(request).await.expect("router responds")
    }

    #[tokio::test]
    async fn import_export_renders_multipart_file_form() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let response = import_export(
            CandidateListImportPath { list_id: list.id },
            Context::new_test_without_db(),
            store,
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
        let csrf_token = context.session.csrf_token.clone();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store.clone(),
            Ok(FileForm {
                csrf_token,
                file_name: Some("invalid.csv".to_string()),
                file_data: Some(Bytes::from(invalid_csv())),
            }),
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
        let csrf_token = context.session.csrf_token.clone();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store,
            Ok(FileForm {
                csrf_token,
                file_name: Some("validation-errors.csv".to_string()),
                file_data: Some(Bytes::from(csv_with_multiple_validation_errors())),
            }),
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

    #[tokio::test]
    async fn import_candidate_list_oversized_file_renders_inline_error() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        // Use a tiny body limit so we can trigger PAYLOAD_TOO_LARGE without
        // generating MAX_IMPORT_SIZE_BYTES of data. The translation message
        // still reports the production limit (5 MB).
        let body = multipart_body("csrf-token", &"a".repeat(2048));
        let response = import_via_router(store, list.id, Body::from(body), 1024).await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert!(body.contains("The selected file is too large. The maximum size is 5 MB."));
        assert_eq!(body.matches("alert alert-warning").count(), 1);

        Ok(())
    }

    #[test]
    fn has_csv_extension_only_accepts_csv() {
        assert!(has_csv_extension("candidates.csv"));
        assert!(has_csv_extension("CANDIDATES.CSV"));
        assert!(has_csv_extension("a.b.csv"));
        assert!(!has_csv_extension(".csv"));
        assert!(!has_csv_extension("candidates.txt"));
        assert!(!has_csv_extension("candidates"));
        assert!(!has_csv_extension("candidates.csv.exe"));
        assert!(!has_csv_extension(""));
    }

    #[tokio::test]
    async fn import_candidate_list_rejects_non_csv_extension() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store,
            Ok(FileForm {
                csrf_token,
                file_name: Some("payload.exe".to_string()),
                file_data: Some(Bytes::from("anything")),
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert!(body.contains("Only CSV files (.csv) are accepted."));
        assert_eq!(body.matches("alert alert-warning").count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_malformed_multipart_renders_inline_error() -> Result<(), AppError>
    {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        // Truncated multipart body — the boundary is opened but the field
        // header is cut off, which causes `next_field()` to return a parse
        // error.
        let truncated = format!("--{TEST_BOUNDARY}\r\nContent-Disposition: form-data; name=\"fiel");
        let response =
            import_via_router(store, list.id, Body::from(truncated), MAX_IMPORT_SIZE_BYTES).await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert!(body.contains("Your file could not be uploaded. Please try again."));
        assert_eq!(body.matches("alert alert-warning").count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn download_csv_template() -> Result<(), AppError> {
        let response = download_import_template(CandidateListImportTemplatePath {}).await?;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("Content-Disposition")
                .unwrap()
                .to_str()
                .unwrap(),
            "attachment; filename=\"kandidatenlijst-export-sjabloon.csv\""
        );

        assert_eq!(
            response
                .headers()
                .get("Content-Type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/csv"
        );
        let body = response_body_string(response).await;
        assert_eq!(body.trim_end_matches('\n'), csv_headers());

        Ok(())
    }
}
