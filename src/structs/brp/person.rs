use chrono::NaiveDate;
use serde::Deserialize;

use crate::{
    common::{Bsn, BsnOrNoneConfirmed, DateOfBirth, DutchAddress, FullName},
    persons::PersonalData,
};

#[derive(Debug, Deserialize)]
#[serde(from = "BrpPersonRaw")]
pub struct BrpPerson {
    pub name: FullName,
    pub personal_data: PersonalData,
    pub address: Option<DutchAddress>,
}

structstruck::strike! {
    #[structstruck::each[derive(Debug, Deserialize)]]
    struct BrpPersonRaw {
        #[serde(rename = "burgerservicenummer")]
        bsn: Option<String>,
        #[serde(rename = "geslacht")]
        gender: Option<struct BrpGender {
            #[serde(rename = "code")]
            gender: String
        }>,
        #[serde(rename = "naam")]
        name: Option<struct BrpName {
            #[serde(rename = "geslachtsnaam")]
            last_name: Option<String>,
            #[serde(rename = "voorvoegsel")]
            last_name_prefix: Option<String>,
            #[serde(rename = "voorletters")]
            initials: Option<String>
        }>,
        #[serde(rename = "geboorte")]
        birth: Option<struct BrpBirth {
            #[serde(rename = "datum")]
            date: Option<struct BrpDate {
                #[serde(rename = "datum")]
                date: Option<String>
            }>
        }>,
        #[serde(rename = "verblijfplaats")]
        place_of_residence: Option<
            #[serde(tag = "type")]
            enum BrpPlaceOfResidence {
            #[serde(rename = "Adres")]
            Address {
                #[serde(rename = "verblijfadres")]
                residence_address: struct BrpAddress {
                    #[serde(rename = "korteStraatnaam")]
                    street_name: Option<String>,
                    #[serde(rename = "huisnummer")]
                    house_number: Option<u32>,
                    #[serde(rename = "huisnummertoevoeging")]
                    house_number_addition: Option<String>,
                    #[serde(rename = "postcode")]
                    postal_code: Option<String>,
                    #[serde(rename = "woonplaats")]
                    place_of_residence: Option<String>,
                }
            },
            #[serde(other)]
            NonDutchAddress
        }>
    }
}

impl From<BrpPersonRaw> for BrpPerson {
    fn from(raw: BrpPersonRaw) -> Self {
        let name = raw
            .name
            .map(|naam| FullName {
                // First name isn't checked, because this does not necessarily need to be the same (roepnaam)
                first_name: None,
                last_name: naam
                    .last_name
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
                last_name_prefix: naam.last_name_prefix.and_then(|s| s.parse().ok()),
                initials: naam
                    .initials
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        let bsn = raw
            .bsn
            .and_then(|s| s.parse::<Bsn>().ok())
            .map(BsnOrNoneConfirmed::Bsn);

        let gender = raw.gender.and_then(|g| g.gender.parse().ok());

        let date_of_birth = raw
            .birth
            .as_ref()
            .and_then(|b| b.date.as_ref())
            .and_then(|d| d.date.as_ref())
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .map(DateOfBirth::from);

        let (address, place_of_residence) = match raw.place_of_residence {
            Some(BrpPlaceOfResidence::Address {
                residence_address: ra,
            }) => {
                let addr = Some(DutchAddress {
                    street_name: ra.street_name.and_then(|s| s.parse().ok()),
                    house_number: ra.house_number.and_then(|s| s.to_string().parse().ok()),
                    house_number_addition: ra.house_number_addition.and_then(|s| s.parse().ok()),
                    locality: ra
                        .place_of_residence
                        .as_deref()
                        .and_then(|s| s.parse().ok()),
                    postal_code: ra.postal_code.and_then(|s| s.parse().ok()),
                    // Known in BRP probably implies known in bag, I guess maybe this could be Some(true), but
                    // I don't think it matters
                    known_in_bag: None,
                });

                // TODO: Is place of residence really the same as locality (above).
                // (though note that above is parsed as `Locality`, and below as `PlaceOfResidence`)
                let por = ra.place_of_residence.and_then(|s| s.parse().ok());

                (addr, por)
            }
            Some(BrpPlaceOfResidence::NonDutchAddress) => {
                // TODO: How to handle this? Set the address to None and conduct an additional BRP check
                // for the Authorised Person?
                tracing::error!("Person has an non-Dutch address");
                (None, None)
            }
            None => {
                tracing::error!("Field 'verblijfplaats' not included");
                (None, None)
            }
        };

        BrpPerson {
            name,
            personal_data: PersonalData {
                gender,
                bsn,
                date_of_birth,
                place_of_residence,
                // TODO: Can country be None here? Because we check with the BRP whether the address is international.
                // If it is, then `address` will be None (since we can't verify international addresses) and we know that
                // instead, it is necesarry to verify the Authorised Person's address
                country: None,
            },
            address,
        }
    }
}
