use crate::structs::candidate_lists::CandidateList;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, Context, HtmlTemplate, Locale, Overlay, PgStore,
    candidate_lists::{
        CSV_HEADERS, CandidateRecordCsv,
        importer::{
            ImportCandidateListError, ImportOutcome, import_candidate_list_csv,
            import_candidate_list_eml,
        },
        pages::{CandidateListImportPath, CandidateListImportTemplatePath},
    },
    core::Csv,
    filters,
    form::FileForm,
    redirect_success, trans,
};

/// Upload body limit, applied via `DefaultBodyLimit` on the route.
pub(crate) const MAX_IMPORT_SIZE_BYTES: usize = 5 * 1024 * 1024;
const MAX_IMPORT_SIZE_MB: usize = MAX_IMPORT_SIZE_BYTES / (1024 * 1024);

#[derive(Template)]
#[template(path = "pg/candidate_lists/pages/import_export.html")]
struct ImportExportTemplate {
    list: CandidateList,
    import_errors: Vec<String>,
    overlay: Overlay,
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
            overlay: Overlay::default(),
        },
        context,
    )
    .into_response()
}

pub async fn import_export(
    CandidateListImportPath { list_id }: CandidateListImportPath,
    context: Context,
    store: PgStore,
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
    store: PgStore,
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

    let Some(format) = ImportFormat::for_upload(import_data.file_name.as_deref()) else {
        return Ok(render_import_export(
            list,
            vec![trans!(
                "candidate_list.import_errors.invalid_file_type",
                context.session.locale
            )],
            context,
        ));
    };

    let Some(file_data) = import_data.file_data else {
        return Ok(render_import_export(
            list.clone(),
            vec![trans!(
                "candidate_list.import_errors.missing_file",
                context.session.locale
            )],
            context,
        ));
    };

    let result = import_file(
        &mut list,
        &store,
        format,
        &file_data,
        context.session.locale,
        import_data.file_name.unwrap_or_default(),
    )
    .await;

    match result {
        Ok(outcome) if outcome.capped => {
            Ok(Redirect::to(&list.import_capped_path().to_string()).into_response())
        }
        Ok(_) => Ok(redirect_success(list.view_path())),
        Err(ImportCandidateListError::App(error)) => Err(error),
        Err(ImportCandidateListError::Messages(messages)) => {
            Ok(render_import_export(list.clone(), messages, context))
        }
    }
}

/// Hand the uploaded bytes to the parser for their format.
async fn import_file(
    list: &mut CandidateList,
    store: &PgStore,
    format: ImportFormat,
    file_data: &[u8],
    locale: Locale,
    file_name: String,
) -> Result<ImportOutcome, ImportCandidateListError> {
    let file_size = file_data.len();

    match format {
        ImportFormat::Csv => {
            import_candidate_list_csv(list, store, file_data, locale, file_name, file_size).await
        }
        ImportFormat::Eml => {
            import_candidate_list_eml(list, store, file_data, locale, file_name, file_size).await
        }
    }
}

/// The file format of an upload, picked from its file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportFormat {
    Csv,
    /// An EML 2.10 nomination, the document this application also exports.
    Eml,
}

impl ImportFormat {
    /// The format of an upload. A file without a name is treated as CSV, the
    /// default format.
    fn for_upload(file_name: Option<&str>) -> Option<Self> {
        match file_name {
            None => Some(ImportFormat::Csv),
            Some(name) => ImportFormat::from_file_name(name),
        }
    }

    /// The upload field's HTML `accept` hint is browser-side only. Pick the
    /// parser from the extension (case-insensitive) and reject anything that is
    /// neither a CSV nor an XML file.
    fn from_file_name(name: &str) -> Option<Self> {
        let lowered = name.to_ascii_lowercase();

        if has_extension(&lowered, ".csv") {
            Some(ImportFormat::Csv)
        } else if has_extension(&lowered, ".xml") {
            Some(ImportFormat::Eml)
        } else {
            None
        }
    }
}

/// A name ends in the extension and is more than the extension alone.
fn has_extension(lowered_name: &str, extension: &str) -> bool {
    lowered_name.ends_with(extension) && lowered_name.len() > extension.len()
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
        AppState, Locale, Session,
        structs::candidate_lists::CandidateListId,
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
        store: PgStore,
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

        let session = Session::new_test_with_locale(Locale::En);
        request.extensions_mut().insert(session);
        request.extensions_mut().insert(store);

        app.oneshot(request).await.expect("router responds")
    }

    #[tokio::test]
    async fn import_export_renders_multipart_file_form() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token().0.clone();

        let response =
            import_export(CandidateListImportPath { list_id: list.id }, context, store).await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("type=\"file\""));
        assert!(body.contains("name=\"file_data\""));
        assert!(body.contains("data-import-file-field"));
        assert!(body.contains("data-import-file-trigger"));
        assert!(body.contains("formenctype=\"multipart/form-data\""));
        // The CSRF guard reads the token for multipart POSTs from the form's
        // action query string, not the body.
        assert!(body.contains(&format!("?csrf_token={csrf_token}\"")));
        assert!(!body.contains("one-click-upload"));

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_invalid_csv_renders_error() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store.clone(),
            Ok(FileForm {
                file_name: Some("invalid.csv".to_string()),
                file_data: Some(Bytes::from(invalid_csv())),
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert!(body.contains("The candidate on line 2 could not be imported:"));
        assert!(body.contains("has 2 columns, but earlier rows have 22"));
        assert_eq!(body.matches("alert alert-warning").count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_renders_multiple_validation_errors() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store,
            Ok(FileForm {
                file_name: Some("validation-errors.csv".to_string()),
                file_data: Some(Bytes::from(csv_with_multiple_validation_errors())),
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert_eq!(
            body.matches("The candidate on line 2 could not be imported.")
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

    #[tokio::test]
    async fn import_candidate_list_capped_redirects_with_warning() -> Result<(), AppError> {
        use crate::MAX_CANDIDATES;

        let store = PgStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();

        let mut csv = format!("{}\r\n", csv_headers());
        for index in 0..(MAX_CANDIDATES + 5) {
            csv.push_str(&format!(
                "H.A.H.A.,Henk,,Jansen{index},Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
            ));
        }

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store,
            Ok(FileForm {
                file_name: Some("candidates.csv".to_string()),
                file_data: Some(Bytes::from(csv)),
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains("import_capped=true"));

        Ok(())
    }

    const CSV_HEADER: &str = include_str!("../testdata/csv_header.csv");
    const NOMINATION: &str = include_str!("../testdata/nomination.eml.xml");
    const POLLING_STATIONS: &str = include_str!("../testdata/polling_stations.eml.xml");

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
            "JD,Henk,,Jansen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1000,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        )
    }

    #[tokio::test]
    async fn import_candidate_list_oversized_file_renders_inline_error() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
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
    fn import_format_follows_the_file_extension() {
        assert_eq!(
            ImportFormat::from_file_name("candidates.csv"),
            Some(ImportFormat::Csv)
        );
        assert_eq!(
            ImportFormat::from_file_name("CANDIDATES.CSV"),
            Some(ImportFormat::Csv)
        );
        assert_eq!(
            ImportFormat::from_file_name("a.b.csv"),
            Some(ImportFormat::Csv)
        );
        assert_eq!(
            ImportFormat::from_file_name("eml210.eml.xml"),
            Some(ImportFormat::Eml)
        );
        assert_eq!(
            ImportFormat::from_file_name("EML210.EML.XML"),
            Some(ImportFormat::Eml)
        );
        assert_eq!(ImportFormat::from_file_name(".csv"), None);
        assert_eq!(ImportFormat::from_file_name(".xml"), None);
        assert_eq!(ImportFormat::from_file_name("candidates.txt"), None);
        assert_eq!(ImportFormat::from_file_name("candidates"), None);
        assert_eq!(ImportFormat::from_file_name("candidates.csv.exe"), None);
        assert_eq!(ImportFormat::from_file_name(""), None);
    }

    #[tokio::test]
    async fn import_candidate_list_imports_an_eml_file() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store.clone(),
            Ok(FileForm {
                file_name: Some("eml210.eml.xml".to_string()),
                file_data: Some(Bytes::from_static(NOMINATION.as_bytes())),
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(store.get_candidate_list(list.id)?.candidates.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_rejects_a_non_nomination_eml_file() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store.clone(),
            Ok(FileForm {
                file_name: Some("polling-stations.eml.xml".to_string()),
                file_data: Some(Bytes::from_static(POLLING_STATIONS.as_bytes())),
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert!(body.contains("not an EML 210 candidate nomination"));
        assert_eq!(body.matches("alert alert-warning").count(), 1);
        assert!(store.get_candidate_list(list.id)?.candidates.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_rejects_unsupported_extension() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = import_candidate_list(
            CandidateListImportPath { list_id: list.id },
            context,
            store,
            Ok(FileForm {
                file_name: Some("payload.exe".to_string()),
                file_data: Some(Bytes::from("anything")),
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("Import failed"));
        assert!(body.contains("Only CSV files (.csv) and EML files (.xml) are accepted."));
        assert_eq!(body.matches("alert alert-warning").count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn import_candidate_list_malformed_multipart_renders_inline_error() -> Result<(), AppError>
    {
        let store = PgStore::new_for_test();
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
        // The template is exported with a UTF-8 BOM and `;` delimiter for Excel.
        let body = response_body_string(response).await;
        assert_eq!(
            body.trim_end_matches('\n'),
            format!("\u{feff}{}", csv_headers().replace(',', ";"))
        );

        Ok(())
    }
}
