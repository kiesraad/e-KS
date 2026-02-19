use serde::{Deserialize, Serialize};

use crate::{
    common::{Bsn, CountryCode, Date, FirstName, FullName, Gender, PlaceOfResidence, UtcDateTime},
    persons::PersonId,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInfo {
    pub person_id: PersonId,
    pub name: FullName,
    pub first_name: Option<FirstName>,
    pub gender: Option<Gender>,
    pub bsn: Option<Bsn>,
    pub no_bsn_confirmed: bool,
    pub date_of_birth: Option<Date>,
    pub place_of_residence: Option<PlaceOfResidence>,
    pub country_of_residence: Option<CountryCode>,
    pub updated_at: UtcDateTime,
}
