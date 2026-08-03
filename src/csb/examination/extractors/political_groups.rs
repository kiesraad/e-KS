use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{
    AppError, CsbStore, CsbStoreData, StreamId,
    store::StoreRegistry,
    structs::{common::FullName, political_groups::PoliticalGroup},
};

pub struct CsbPoliticalGroup {
    pub political_group: PoliticalGroup,
    pub stream_id: StreamId,
    pub is_examination_finished: bool,
    pub omission_count: usize,
    pub first_candidate_name: Option<FullName>,
}

impl CsbPoliticalGroup {
    pub fn new_from_csb_store(store: &CsbStore) -> Self {
        CsbPoliticalGroup {
            political_group: store.get_political_group(crate::csb::WithCorrections::All),
            stream_id: store.stream_id,
            is_examination_finished: store.is_examination_finished(),
            omission_count: store.get_omission_count(),
            first_candidate_name: store.get_first_candidate_name(crate::csb::WithCorrections::All),
        }
    }

    pub fn csb_display_name(&self) -> String {
        self.political_group
            .csb_display_name(self.first_candidate_name.as_ref())
    }
}

/// Extracts all imported political groups visible to the CSB scope.
pub struct CsbPoliticalGroups(pub Vec<CsbPoliticalGroup>);

impl<S> FromRequestParts<S> for CsbPoliticalGroups
where
    S: Send + Sync,
    StoreRegistry<CsbStoreData>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let registry = StoreRegistry::<CsbStoreData>::from_ref(state);

        let mut political_groups = Vec::new();
        for store in registry.stores_by_scope().await? {
            political_groups.push(CsbPoliticalGroup::new_from_csb_store(&store));
        }

        Ok(CsbPoliticalGroups(political_groups))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};

    use crate::{
        AppState, CsbEvent, ElectionConfig, PgStoreData, structs::list_designation::ListDesignation,
    };

    /// Persist a CSB stream carrying a single import event in the (in-memory)
    /// test registry, returning its `stream_id`.
    async fn seed_csb_store(state: &AppState, election: ElectionConfig) -> StreamId {
        let stream_id = StreamId::new();
        let store = state
            .csb_store_for_stream(stream_id, election)
            .await
            .unwrap();
        store
            .update(CsbEvent::Import {
                hash: [0u8; 32],
                source_stream_id: StreamId::new(),
                snapshot: Box::new(PgStoreData::default()),
            })
            .await
            .unwrap();
        stream_id
    }

    fn empty_parts() -> axum::http::request::Parts {
        Request::builder()
            .uri("/csb/examination")
            .body(Body::empty())
            .unwrap()
            .into_parts()
            .0
    }

    #[tokio::test]
    async fn returns_every_csb_scoped_political_group() {
        let state = AppState::new_for_tests().await;
        let first = seed_csb_store(&state, ElectionConfig::EK27).await;
        let second = seed_csb_store(&state, ElectionConfig::EK27).await;

        let mut parts = empty_parts();
        let CsbPoliticalGroups(groups) = CsbPoliticalGroups::from_request_parts(&mut parts, &state)
            .await
            .unwrap();

        assert_eq!(groups.len(), 2);
        let stream_ids: Vec<_> = groups.iter().map(|g| g.stream_id).collect();
        assert!(stream_ids.contains(&first));
        assert!(stream_ids.contains(&second));
    }

    #[tokio::test]
    async fn returns_empty_when_nothing_imported() {
        let state = AppState::new_for_tests().await;

        let mut parts = empty_parts();
        let CsbPoliticalGroups(groups) = CsbPoliticalGroups::from_request_parts(&mut parts, &state)
            .await
            .unwrap();

        assert!(groups.is_empty());
    }

    #[test]
    fn csb_display_name_returns_display_name_for_normal_list() {
        let group = CsbPoliticalGroup {
            political_group: PoliticalGroup {
                display_name: Some("Kiesraad Demo".parse().unwrap()),
                list_designation: Some(ListDesignation::Standalone),
                ..Default::default()
            },
            stream_id: StreamId::new(),
            is_examination_finished: false,
            omission_count: 0,
            first_candidate_name: None,
        };

        assert_eq!(group.csb_display_name(), "Kiesraad Demo");
    }

    #[test]
    fn csb_display_name_blank_list_with_candidate_uses_first_candidate_name() {
        let group = CsbPoliticalGroup {
            political_group: PoliticalGroup {
                list_designation: Some(ListDesignation::Blank),
                ..Default::default()
            },
            stream_id: StreamId::new(),
            is_examination_finished: false,
            omission_count: 0,
            first_candidate_name: Some(FullName {
                last_name: "Jansen".parse().unwrap(),
                initials: "A.B.".parse().unwrap(),
                ..Default::default()
            }),
        };

        assert_eq!(group.csb_display_name(), "Blanco (Jansen, A.B.)");
    }

    #[test]
    fn csb_display_name_blank_list_without_candidates_uses_blanco_fallback() {
        let group = CsbPoliticalGroup {
            political_group: PoliticalGroup {
                list_designation: Some(ListDesignation::Blank),
                ..Default::default()
            },
            stream_id: StreamId::new(),
            is_examination_finished: false,
            omission_count: 0,
            first_candidate_name: None,
        };

        assert_eq!(group.csb_display_name(), "Blanco");
    }
}
