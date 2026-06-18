use crate::{
    AppError, AppEvent, AppStore,
    common::{
        Address, FullName, InternationalAddress, InternationalPostalCode, PostalCode, Problematic,
        Problems, Severity,
    },
    id_newtype,
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct ListSubmitterId);

#[derive(Default, Debug, Clone)]
pub struct ListSubmitterData {
    pub name: FullName,
    pub address: InternationalAddress,
}

impl From<ListSubmitterData> for ListSubmitter {
    fn from(value: ListSubmitterData) -> Self {
        let is_dutch = value
            .address
            .country
            .as_ref()
            .is_none_or(|country| country.is_nl());

        let address = if is_dutch {
            try_into_dutch_address(&value.address)
                .map(Address::Dutch)
                .unwrap_or(Address::International(value.address))
        } else {
            Address::International(value.address)
        };

        ListSubmitter {
            name: value.name,
            address,
            ..Default::default()
        }
    }
}

impl From<ListSubmitter> for ListSubmitterData {
    fn from(value: ListSubmitter) -> Self {
        let address = match value.address {
            Address::Dutch(address) => InternationalAddress {
                street_name: address.street_name,
                house_number: address.house_number,
                house_number_addition: address.house_number_addition,
                locality: address.locality,
                state_or_province: None,
                postal_code: address.postal_code.map(|postal_code| {
                    postal_code
                        .to_string()
                        .parse::<InternationalPostalCode>()
                        .expect("dutch postal code must fit international postal code")
                }),
                country: None,
            },
            Address::International(address) => address,
        };

        ListSubmitterData {
            name: value.name,
            address,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListSubmitter {
    pub id: ListSubmitterId,
    pub name: FullName,
    pub address: Address,
    #[serde(skip)]
    pub is_substitute: bool,
}

impl Problematic<()> for ListSubmitter {
    fn get_problems(&self, _: ()) -> Problems {
        let severity = if self.is_substitute {
            Severity::Info
        } else {
            Severity::Error
        };

        if self.is_empty() && !self.is_substitute {
            return Problems::new_empty(); // error gets returned in general problems
        }

        Problems::merge(vec![
            self.name.get_problems(severity),
            self.address.get_problems(severity),
        ])
    }
}

impl ListSubmitter {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.address.is_empty()
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateListSubmitter(self.clone()))
            .await
    }

    pub async fn create_substitute(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateSubstituteSubmitter(self.clone()))
            .await
    }

    pub async fn update_substitute(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateSubstituteSubmitter(self.clone()))
            .await
    }

    pub async fn delete_substitute(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteSubstituteSubmitter {
                substitute_submitter_id: self.id,
            })
            .await
    }

    pub fn address_line_1(&self) -> String {
        self.address.address_line_1().unwrap_or_default()
    }

    pub fn address_line_2(&self) -> String {
        self.address.address_line_2().unwrap_or_default()
    }
}

fn try_into_dutch_address(address: &InternationalAddress) -> Option<crate::common::DutchAddress> {
    Some(crate::common::DutchAddress {
        street_name: address.street_name.clone(),
        house_number: address.house_number.clone(),
        house_number_addition: address.house_number_addition.clone(),
        locality: address.locality.clone(),
        postal_code: address
            .postal_code
            .as_ref()
            .map(|postal_code| postal_code.to_string().parse::<PostalCode>())
            .transpose()
            .ok()?,
        known_in_bag: None,
    })
}

#[cfg(test)]
mod tests {
    use crate::common::{EmptyAddressProblems, InfoProblems, PotentialProblems};

    use super::*;

    fn incomplete_submitter(is_substitute: bool) -> ListSubmitter {
        ListSubmitter {
            id: ListSubmitterId::new(),
            name: FullName {
                last_name_prefix: Some("van".parse().unwrap()),
                ..Default::default()
            },
            address: Address::Dutch(crate::common::DutchAddress::default()),
            is_substitute,
        }
    }

    #[test]
    fn main_submitter_problems_use_error_severity() {
        let problems = incomplete_submitter(false).get_problems(());

        assert!(
            problems
                .potential_problems
                .contains(&PotentialProblems::NoLastName(Severity::Error))
        );
        assert!(problems.potential_problems.iter().any(|pp| match pp {
            PotentialProblems::IncompleteAddress {
                severity: Severity::Error,
                problems,
            } => {
                problems.contains(&EmptyAddressProblems::StreetName)
            }
            _ => false,
        }));
        assert!(problems.info_problems.is_empty());
    }

    #[test]
    fn international_submitter_address_is_not_bag_checked() {
        let data = ListSubmitterData {
            name: FullName {
                last_name: "Bos".parse().expect("last name"),
                initials: "E.F.".parse().expect("initials"),
                ..Default::default()
            },
            address: InternationalAddress {
                street_name: Some("Downing Street".parse().expect("street name")),
                house_number: Some("10".parse().expect("house number")),
                house_number_addition: None,
                locality: Some("London".parse().expect("locality")),
                state_or_province: None,
                postal_code: Some("SW1A 2AA".parse().expect("postal code")),
                country: Some("GB".parse().expect("country code")),
            },
        };

        let submitter = ListSubmitter::from(data);

        assert!(matches!(submitter.address, Address::International(_)));
        assert!(
            !submitter
                .get_problems(())
                .potential_problems
                .contains(&PotentialProblems::UnknownAddress)
        );
    }

    #[test]
    fn substitute_submitter_problems_use_info_severity() {
        let problems = incomplete_submitter(true).get_problems(());

        assert!(problems.info_problems.contains(&InfoProblems::NoLastName));
        assert!(problems.info_problems.contains(&InfoProblems::NoLastName));
        assert!(problems.info_problems.iter().any(|pp| match pp {
            InfoProblems::IncompleteAddress { problems } => {
                problems.contains(&EmptyAddressProblems::StreetName)
            }
            _ => false,
        }));
        assert!(problems.potential_problems.is_empty());
    }
}
