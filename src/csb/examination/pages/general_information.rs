use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, CsbStore, HtmlTemplate,
    csb::{
        Omission,
        examination::{extractors::CsbPoliticalGroup, pages::CsbGeneralInformationPath},
    },
    filters,
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
};

#[derive(Template)]
#[template(path = "csb/examination/pages/general_information.html")]
struct CsbGeneralInformationTemplate {
    political_group: CsbPoliticalGroup,
    name_authorisations: Vec<NameAuthorisation>,
    list_submitter: ListSubmitter,
    substitute_submitters: Vec<ListSubmitter>,
    political_group_omissions: Vec<Omission>,
    restoration_count: usize,
}

/// Render the placeholder general information (basisgegevens) page for a
/// single political group under examination.
pub async fn overview(
    _: CsbGeneralInformationPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let name_authorisations = store.get_name_authorisations();
    let list_submitter = store.get_list_submitter();
    let substitute_submitters = store.get_substitute_submitters();
    let political_group_omissions = store.get_political_group_omissions();

    Ok(HtmlTemplate(
        CsbGeneralInformationTemplate {
            political_group,
            name_authorisations,
            list_submitter,
            substitute_submitters,
            political_group_omissions,
            restoration_count: store.get_omission_count(),
        },
        context,
    )
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::test_utils::{response_body_string, sample_political_group};

    #[tokio::test]
    async fn renders_section_headings_and_registered_designation() {
        let store = CsbStore::new_for_test();
        store.set_political_group(sample_political_group());
        let stream_id = store.stream_id;

        let response = overview(
            CsbGeneralInformationPath { stream_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The political group section and the imported registered designation
        // (the test session uses the English locale).
        assert!(body.contains("Political group information"));
        assert!(body.contains("Kiesraad Demo"));
        // The substitutes section heading is always present.
        assert!(body.contains("Substitute submitters data"));
    }

    #[tokio::test]
    async fn renders_without_imported_data() {
        // A fresh store has no imported political group or substitutes.
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let response = overview(
            CsbGeneralInformationPath { stream_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("General Information"));
    }

    #[tokio::test]
    async fn renders_added_political_group_omissions_as_badges() {
        use crate::csb::OmissionCategory;

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        Omission::new(
            OmissionCategory::PoliticalGroup,
            "Deposit missing".to_string(),
            "The deposit has not been paid.".to_string(),
            String::new(),
        )
        .create(&store)
        .await
        .unwrap();

        let response = overview(
            CsbGeneralInformationPath { stream_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("omission-badge"));
        // The badge shows the short title, not the long description.
        assert!(body.contains("Deposit missing"));
        assert!(!body.contains("The deposit has not been paid."));
        // A recoverable omission is not highlighted as an error.
        assert!(!body.contains("omission-badge-unrecoverable"));
    }

    #[tokio::test]
    async fn renders_non_recoverable_omission_as_error() {
        use crate::csb::OmissionCategory;

        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let mut omission = Omission::new(
            OmissionCategory::PoliticalGroup,
            "Unregistered designation".to_string(),
            "The designation is not registered.".to_string(),
            String::new(),
        );
        omission.recoverable = false;
        omission.create(&store).await.unwrap();

        let response = overview(
            CsbGeneralInformationPath { stream_id },
            CsbContext::new_test(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("omission-badge-unrecoverable"));
    }
}
