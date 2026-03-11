use crate::{
    AppError, AppEvent, AppStore,
    common::{Address, FullName, InternationalAddress, InternationalPostalCode, PostalCode},
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
            .is_none_or(|country| country.as_str() == "NL");

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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ListSubmitter {
    pub id: ListSubmitterId,
    pub name: FullName,
    pub address: Address,
}

impl ListSubmitter {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete() && self.address.is_complete()
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateListSubmitter(self.clone()))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateListSubmitter(self.clone()))
            .await
    }

    pub async fn delete(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteListSubmitter {
                list_submitter_id: self.id,
            })
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
    })
}
