use serde::{Deserialize, Serialize};

use crate::{
    Bsn, CountryCode, Date, FirstName, FullName, PlaceOfResidence, UtcDateTime,
    persons::{Gender, PersonId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
