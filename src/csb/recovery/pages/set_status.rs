use axum::{extract::Query, response::Response};
use serde::Deserialize;

use crate::{
    AppError, CsbContext, CsbStore, Form, QueryParamState,
    csb::{examination::extractors::CsbPoliticalGroup, recovery::paths::CsbSetOmissionStatusPath},
    structs::csb::{CsbPhase, OmissionStatus},
};

#[derive(Deserialize)]
pub struct OmissionStatusForm {
    status: OmissionStatusFormValue,
}

/// The submitted decision. A dedicated form enum keeps the kebab-case form
/// values separate from the persisted event encoding of [`OmissionStatus`].
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OmissionStatusFormValue {
    Recovered,
    NotRecovered,
}

/// Record whether an omission was recovered, returning to the page the
/// control was on. Irreparable omissions are rejected by
/// [`Omission::set_status`](crate::structs::csb::Omission).
pub async fn set_status(
    CsbSetOmissionStatusPath { omission_id, .. }: CsbSetOmissionStatusPath,
    _context: CsbContext,
    store: CsbStore,
    Query(query): Query<QueryParamState>,
    Form(form): Form<OmissionStatusForm>,
) -> Result<Response, AppError> {
    let omission = store.get_omission(omission_id)?;
    let status = match form.status {
        OmissionStatusFormValue::Recovered => OmissionStatus::Recovered,
        OmissionStatusFormValue::NotRecovered => OmissionStatus::NotRecovered,
    };
    omission.set_status(&store, status).await?;

    let political_group =
        CsbPoliticalGroup::new_from_csb_store(&store).with_mode(CsbPhase::Recovery);
    Ok(query.redirect_or(political_group.all_restorations_path()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};

    use crate::structs::csb::{OmissionCategory, sample_omission};

    fn form(value: OmissionStatusFormValue) -> Form<OmissionStatusForm> {
        Form(OmissionStatusForm { status: value })
    }

    #[tokio::test]
    async fn records_the_decision_and_redirects_to_the_todo_page() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let omission = sample_omission(OmissionCategory::PoliticalGroup);
        omission.create(&store).await.unwrap();

        let response = set_status(
            CsbSetOmissionStatusPath {
                stream_id,
                omission_id: omission.id,
            },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::default()),
            form(OmissionStatusFormValue::Recovered),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            store.get_omission(omission.id).unwrap().status,
            OmissionStatus::Recovered
        );
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains(&format!("/csb/recovery/{stream_id}/omissions")));
    }

    #[tokio::test]
    async fn honours_the_redirect_to() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let omission = sample_omission(OmissionCategory::PoliticalGroup);
        omission.create(&store).await.unwrap();

        let response = set_status(
            CsbSetOmissionStatusPath {
                stream_id,
                omission_id: omission.id,
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::redirect_to("/back/here".to_string())),
            form(OmissionStatusFormValue::NotRecovered),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("/back/here"));
    }

    #[tokio::test]
    async fn rejects_irreparable_omissions() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;
        let mut omission = sample_omission(OmissionCategory::PoliticalGroup);
        omission.recoverable = false;
        omission.create(&store).await.unwrap();

        let result = set_status(
            CsbSetOmissionStatusPath {
                stream_id,
                omission_id: omission.id,
            },
            CsbContext::new_test(),
            store.clone(),
            Query(QueryParamState::default()),
            form(OmissionStatusFormValue::Recovered),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(
            store.get_omission(omission.id).unwrap().status,
            OmissionStatus::Pending
        );
    }

    #[tokio::test]
    async fn errors_for_an_unknown_omission() {
        let store = CsbStore::new_for_test();
        let stream_id = store.stream_id;

        let result = set_status(
            CsbSetOmissionStatusPath {
                stream_id,
                omission_id: crate::structs::csb::OmissionId::new(),
            },
            CsbContext::new_test(),
            store,
            Query(QueryParamState::default()),
            form(OmissionStatusFormValue::Recovered),
        )
        .await;

        assert!(matches!(result, Err(AppError::GenericNotFound)));
    }
}
