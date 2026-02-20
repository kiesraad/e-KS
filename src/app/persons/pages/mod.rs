use crate::{
    AppError, AppState, QueryParamState,
    persons::{Person, PersonId},
};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

mod create;
mod delete;
mod list;
mod update;
mod update_address;
mod update_representative;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons", rejection(AppError))]
pub struct PersonsPath;

#[derive(TypedPath)]
#[typed_path("/persons/create", rejection(AppError))]
pub struct PersonsCreatePath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/update", rejection(AppError))]
pub struct UpdatePersonPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/delete", rejection(AppError))]
pub struct DeletePersonPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/address", rejection(AppError))]
pub struct UpdatePersonAddressPath {
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/persons/{person_id}/representative", rejection(AppError))]
pub struct UpdateRepresentativePath {
    pub person_id: PersonId,
}

impl Person {
    pub fn list_path() -> impl TypedPath {
        PersonsPath {}
    }

    pub fn highlight_path(&self) -> impl TypedPath {
        PersonsPath {}.with_query_params(QueryParamState::highlight(self.id.into()))
    }

    pub fn highlight_success_path(&self) -> impl TypedPath {
        PersonsPath {}.with_query_params(QueryParamState::highlight_success(self.id.into()))
    }

    pub fn create_path() -> impl TypedPath {
        PersonsCreatePath {}
    }

    pub fn update_path(&self) -> impl TypedPath {
        UpdatePersonPath { person_id: self.id }
    }

    pub fn delete_path(&self) -> impl TypedPath {
        DeletePersonPath { person_id: self.id }
    }

    pub fn update_address_path(&self) -> impl TypedPath {
        UpdatePersonAddressPath { person_id: self.id }
    }

    pub fn update_representative_path(&self) -> impl TypedPath {
        UpdateRepresentativePath { person_id: self.id }
    }

    pub fn after_update_path(&self) -> String {
        if self.lives_in_nl() {
            UpdatePersonAddressPath { person_id: self.id }
                .with_query_params(QueryParamState::success())
                .to_string()
        } else {
            UpdateRepresentativePath { person_id: self.id }
                .with_query_params(QueryParamState::success())
                .to_string()
        }
    }

    pub fn after_create_path(&self) -> String {
        if self.lives_in_nl() {
            UpdatePersonAddressPath { person_id: self.id }
                .with_query_params(QueryParamState::created())
                .to_string()
        } else {
            UpdateRepresentativePath { person_id: self.id }
                .with_query_params(QueryParamState::created())
                .to_string()
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::list_persons)
        .typed_post(create::create_person_submit)
        .typed_get(create::create_person)
        .typed_get(update::update_person)
        .typed_post(update::update_person_submit)
        .typed_get(update_address::update_person_address)
        .typed_post(update_address::update_person_address_submit)
        .typed_get(update_representative::update_representative)
        .typed_post(update_representative::update_representative_submit)
        .typed_post(delete::delete_person)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::CountryCode, persons::PersonId, test_utils::sample_person};

    #[test]
    fn person_paths_match_expected_routes() {
        let person = sample_person(PersonId::new());

        assert_eq!(Person::list_path().to_string(), "/persons");
        assert_eq!(Person::create_path().to_string(), "/persons/create");
        assert_eq!(
            person.update_path().to_string(),
            format!("/persons/{}/update", person.id)
        );
        assert_eq!(
            person.delete_path().to_string(),
            format!("/persons/{}/delete", person.id)
        );
        assert_eq!(
            person.update_address_path().to_string(),
            format!("/persons/{}/address", person.id)
        );
        assert_eq!(
            person.update_representative_path().to_string(),
            format!("/persons/{}/representative", person.id)
        );
    }

    #[test]
    fn person_after_create_path_depends_on_residence() {
        let mut dutch = sample_person(PersonId::new());
        dutch.country_of_residence = Some("NL".parse::<CountryCode>().expect("country code"));
        let mut foreign = sample_person(PersonId::new());
        foreign.country_of_residence = Some("BE".parse::<CountryCode>().expect("country code"));

        let expected_dutch = format!("/persons/{}/address?&initial=true&success=true", dutch.id);
        let expected_foreign = format!(
            "/persons/{}/representative?&initial=true&success=true",
            foreign.id
        );

        assert_eq!(dutch.after_create_path(), expected_dutch);
        assert_eq!(foreign.after_create_path(), expected_foreign);
    }

    #[test]
    fn person_router_builds() {
        let _router = router();
    }
}
