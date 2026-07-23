use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppState;

use super::paths::*;

mod add;
mod create;
mod delete;
mod update;
mod update_address;
mod update_position;
mod update_representative;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(add::add_existing_person)
        .typed_post(add::add_person_to_candidate_list)
        .typed_get(update_position::update_candidate_position)
        .typed_post(update_position::update_candidate_position_submit)
        .typed_get(create::create_person_candidate_list)
        .typed_post(create::create_person_candidate_list_submit)
        .typed_get(update_address::update_person_address)
        .typed_post(update_address::update_person_address_submit)
        .typed_get(update_representative::update_representative)
        .typed_post(update_representative::update_representative_submit)
        .typed_get(update::update_person)
        .typed_post(update::update_person_submit)
        .typed_get(delete::delete_person_confirm)
        .typed_post(delete::delete_person)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        candidate_lists::CandidateListId, candidates::Candidate, common::CountryCode,
        persons::PersonId, test_utils::sample_person,
    };

    #[test]
    fn candidate_paths_match_expected_routes() {
        let list_id = CandidateListId::new();
        let person = sample_person(PersonId::new());
        let candidate = Candidate {
            list_id,
            position: 1,
            person,
        };

        assert_eq!(
            candidate.update_position_path().to_string(),
            format!(
                "/candidate-lists/{}/reorder/{}",
                candidate.list_id, candidate.person.id
            )
        );
        assert_eq!(
            candidate.update_path().to_string(),
            format!(
                "/candidate-lists/{}/update/{}",
                candidate.list_id, candidate.person.id
            )
        );
        assert_eq!(
            candidate.update_address_path().to_string(),
            format!(
                "/candidate-lists/{}/address/{}",
                candidate.list_id, candidate.person.id
            )
        );
        assert_eq!(
            candidate.update_representative_path().to_string(),
            format!(
                "/candidate-lists/{}/representative/{}",
                candidate.list_id, candidate.person.id
            )
        );
        assert_eq!(
            candidate.delete_path().to_string(),
            format!(
                "/candidate-lists/{}/delete/{}",
                candidate.list_id, candidate.person.id
            )
        );
    }

    #[test]
    fn candidate_after_create_path_depends_on_residence() {
        let list_id = CandidateListId::new();
        let mut dutch_person = sample_person(PersonId::new());
        dutch_person.personal_data.country =
            Some("NL".parse::<CountryCode>().expect("country code"));
        let dutch_candidate = Candidate {
            list_id,
            position: 1,
            person: dutch_person,
        };

        let mut foreign_person = sample_person(PersonId::new());
        foreign_person.personal_data.country =
            Some("BE".parse::<CountryCode>().expect("country code"));
        let foreign_candidate = Candidate {
            list_id,
            position: 1,
            person: foreign_person,
        };

        let expected_dutch = format!(
            "/candidate-lists/{}/address/{}?&initial=true&success=true",
            dutch_candidate.list_id, dutch_candidate.person.id
        );
        let expected_foreign = format!(
            "/candidate-lists/{}/representative/{}?&initial=true&success=true",
            foreign_candidate.list_id, foreign_candidate.person.id
        );

        assert_eq!(
            dutch_candidate.after_create_path().to_string(),
            expected_dutch
        );
        assert_eq!(
            foreign_candidate.after_create_path().to_string(),
            expected_foreign
        );
    }

    #[test]
    fn candidate_router_builds() {
        let _router = router();
    }
}
