//! Store-backed operations for omissions.

use std::collections::BTreeMap;

use crate::{
    AnyLocale, AppError, CsbEvent, CsbStore, CsbStoreData, ElectionConfig, ElectoralDistrict,
    models::{
        i1::{DistrictLists, SubmittedList},
        i4::OmissionGroup,
    },
    projection::WithCorrections,
    store::StoreRegistry,
    structs::csb::{Omission, OmissionCategory},
};

const ALL_DISTRICTS: &str = "alle kieskringen";

/// The submitted candidate lists per electoral district, as listed in the
/// "Kandidatenlijsten" section of model I 1: one row per list and district,
/// with the political group's designation, its first candidate and the number
/// of names on the list.
///
/// A list that covers several districts contributes a row to each of them.
/// Districts come out in electoral-district order, the rows within a district
/// in store scope order and then by list creation date.
pub async fn submitted_lists(
    registry: &StoreRegistry<CsbStoreData>,
    election: &ElectionConfig,
) -> Result<Vec<DistrictLists>, AppError> {
    let mut by_district: BTreeMap<ElectoralDistrict, Vec<SubmittedList>> = BTreeMap::new();
    for store in registry.stores_by_scope().await? {
        for (district, list) in store_submitted_lists(&store) {
            by_district.entry(district).or_default().push(list);
        }
    }

    Ok(by_district
        .into_iter()
        .map(|(district, lists)| DistrictLists {
            electoral_district: district_label(district, election),
            lists,
        })
        .collect())
}

/// The rows one political group contributes to the submitted lists section,
/// paired with the district they belong to.
fn store_submitted_lists(store: &CsbStore) -> Vec<(ElectoralDistrict, SubmittedList)> {
    let designation = store.get_display_name(WithCorrections::All);
    let mut lists = store.get_candidate_lists(WithCorrections::All);
    lists.sort_unstable_by_key(|list| list.created_at);

    let mut rows = Vec::new();
    for list in lists {
        let first_candidate_name = list
            .candidates
            .first()
            .and_then(|id| store.get_person(*id, WithCorrections::All))
            .map(|person| person.name.display())
            .unwrap_or_default();

        for district in &list.electoral_districts {
            rows.push((
                *district,
                SubmittedList {
                    designation: designation.clone(),
                    first_candidate_name: first_candidate_name.clone(),
                    candidate_count: list.candidates.len(),
                },
            ));
        }
    }

    rows
}

/// A single district as printed after the "Kieskring" label: its number and
/// title, e.g. `1 (Groningen)`. Elections with one district have no meaningful
/// district number, so those print the title alone.
fn district_label(district: ElectoralDistrict, election: &ElectionConfig) -> String {
    let title = district.title(AnyLocale::Nl);
    if election.has_only_one_district() {
        title.to_string()
    } else {
        format!("{} ({})", district.region_number(), title)
    }
}

/// The recoverable omissions of every political group, as the model input
/// groups shared by models I 1 and I 4: one group per political group and
/// electoral district, in store scope order.
pub async fn found_omissions(
    registry: &StoreRegistry<CsbStoreData>,
    election: &ElectionConfig,
) -> Result<Vec<OmissionGroup>, AppError> {
    let mut found_omissions = Vec::new();
    for store in registry.stores_by_scope().await? {
        let recoverable = store.get_recoverable_omissions();
        if recoverable.is_empty() {
            continue;
        }

        let mut by_district: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for omission in recoverable {
            let district = omission.category.electoral_district(&store, election)?;
            by_district
                .entry(district)
                .or_default()
                .push(omission.description.to_string());
        }

        let designation = store.get_display_name(WithCorrections::All);
        for (district, descriptions) in by_district {
            found_omissions.push(OmissionGroup {
                designation: designation.clone(),
                electoral_district: district,
                omission_descriptions: descriptions,
            });
        }
    }

    Ok(found_omissions)
}

impl OmissionCategory {
    /// Returns the electoral district string for use in models I 1 and I 4.
    pub fn electoral_district(
        &self,
        store: &CsbStore,
        election: &ElectionConfig,
    ) -> Result<String, AppError> {
        match self {
            OmissionCategory::PoliticalGroup => Ok(ALL_DISTRICTS.to_string()),
            OmissionCategory::CandidateList(lists) | OmissionCategory::Candidate { lists, .. } => {
                let mut districts: Vec<ElectoralDistrict> = Vec::new();
                for id in lists {
                    let list = store
                        .get_candidate_list(*id, crate::projection::WithCorrections::All)
                        .ok_or(AppError::GenericNotFound)?;
                    for d in list.electoral_districts {
                        if !districts.contains(&d) {
                            districts.push(d);
                        }
                    }
                }
                Ok(format_districts(&districts, election))
            }
            OmissionCategory::DeclarationsOfSupport(districts) => {
                Ok(format_districts(districts, election))
            }
        }
    }
}

fn format_districts(districts: &[ElectoralDistrict], election: &ElectionConfig) -> String {
    let all_districts = election.electoral_districts();
    if districts.is_empty() || all_districts.iter().all(|d| districts.contains(d)) {
        ALL_DISTRICTS.to_string()
    } else {
        let mut sorted = districts.to_vec();
        sorted.sort_by_key(|d| d.region_number());
        let parts: Vec<String> = sorted
            .iter()
            .map(|d| format!("{} ({})", d.region_number(), d.title(AnyLocale::Nl)))
            .collect();
        format!("kieskring {}", parts.join(", "))
    }
}

impl Omission {
    pub async fn create(&self, store: &CsbStore) -> Result<(), AppError> {
        store.update(CsbEvent::CreateOmission(self.clone())).await
    }

    pub async fn update(&self, store: &CsbStore) -> Result<(), AppError> {
        store.update(CsbEvent::UpdateOmission(self.clone())).await
    }

    pub async fn delete(&self, store: &CsbStore) -> Result<(), AppError> {
        store
            .update(CsbEvent::DeleteOmission {
                omission_id: self.id,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectionConfig, ElectoralDistrict,
        structs::{
            candidate_lists::{CandidateList, CandidateListId},
            persons::PersonId,
        },
        test_utils::sample_person,
    };

    const EK: ElectionConfig = ElectionConfig::EK27;

    fn store_with_list(districts: Vec<ElectoralDistrict>) -> (CsbStore, CandidateListId) {
        let store = CsbStore::new_for_test();
        let list = CandidateList {
            electoral_districts: districts,
            ..Default::default()
        };
        let id = list.id;
        store.add_candidate_list(list);
        (store, id)
    }

    #[test]
    fn political_group_maps_to_all_districts() {
        let store = CsbStore::new_for_test();
        assert_eq!(
            OmissionCategory::PoliticalGroup
                .electoral_district(&store, &EK)
                .unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn candidate_with_all_districts_maps_to_all() {
        let (store, id) = store_with_list(EK.electoral_districts().to_vec());
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };
        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn dos_all_districts_maps_to_all() {
        let store = CsbStore::new_for_test();
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(EK.electoral_districts().to_vec())
                .electoral_district(&store, &EK)
                .unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn dos_one_district() {
        let store = CsbStore::new_for_test();
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(vec![ElectoralDistrict::BO])
                .electoral_district(&store, &EK)
                .unwrap(),
            "kieskring 13 (Bonaire)"
        );
    }

    #[test]
    fn dos_multiple_districts() {
        let store = CsbStore::new_for_test();
        // The districts should be sorted by region number.
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(vec![
                ElectoralDistrict::DR,
                ElectoralDistrict::GR
            ])
            .electoral_district(&store, &EK)
            .unwrap(),
            "kieskring 1 (Groningen), 3 (Drenthe)"
        );
    }

    #[test]
    fn dos_no_districts_maps_to_all() {
        let store = CsbStore::new_for_test();
        // An empty district list is treated as "all districts" in format_districts.
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(vec![])
                .electoral_district(&store, &EK)
                .unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn candidate_with_list_specific_district() {
        let (store, id) = store_with_list(vec![ElectoralDistrict::GR]);
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };
        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "kieskring 1 (Groningen)"
        );
    }

    #[test]
    fn candidate_with_paper_added_list_uses_the_corrected_projection() {
        let store = CsbStore::new_for_test();
        let list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::GR],
            ..Default::default()
        };
        let id = list.id;
        store.set_paper_corrected_candidate_list(list);
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };

        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "kieskring 1 (Groningen)"
        );
    }

    #[test]
    fn candidate_with_corrected_list_uses_the_corrected_districts() {
        let (store, id) = store_with_list(vec![ElectoralDistrict::UT]);
        store.set_paper_corrected_candidate_list(CandidateList {
            id,
            electoral_districts: vec![ElectoralDistrict::GR],
            ..Default::default()
        });
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };

        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "kieskring 1 (Groningen)"
        );
    }

    #[test]
    fn candidate_with_missing_list_returns_error() {
        let store = CsbStore::new_for_test();
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![CandidateListId::new()],
        };
        assert!(category.electoral_district(&store, &EK).is_err());
    }

    #[test]
    fn submitted_list_row_per_district_carries_first_candidate_and_count() {
        let store = CsbStore::new_for_test();
        let person = sample_person(PersonId::new());
        let other = sample_person(PersonId::new());
        store.add_person(person.clone());
        store.add_person(other.clone());
        store.add_candidate_list(CandidateList {
            electoral_districts: vec![ElectoralDistrict::GR, ElectoralDistrict::BO],
            candidates: vec![person.id, other.id],
            ..Default::default()
        });

        let rows = store_submitted_lists(&store);

        // The list covers two districts, so it contributes a row to each.
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, ElectoralDistrict::GR);
        assert_eq!(rows[1].0, ElectoralDistrict::BO);
        for (_, list) in &rows {
            assert_eq!(list.first_candidate_name, person.name.display());
            assert_eq!(list.candidate_count, 2);
        }
    }

    #[test]
    fn submitted_list_without_candidates_has_an_empty_first_candidate() {
        let (store, _) = store_with_list(vec![ElectoralDistrict::BO]);

        let rows = store_submitted_lists(&store);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1.first_candidate_name, "");
        assert_eq!(rows[0].1.candidate_count, 0);
    }

    #[test]
    fn district_label_prefixes_the_district_number() {
        assert_eq!(district_label(ElectoralDistrict::BO, &EK), "13 (Bonaire)");
        assert_eq!(district_label(ElectoralDistrict::GR, &EK), "1 (Groningen)");
    }
}
