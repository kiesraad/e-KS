use serde::{Deserialize, Serialize};

use crate::common::{BsnOrNoneConfirmed, CountryCode, Date, FirstName, Gender, PlaceOfResidence};

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct PersonalData {
    pub first_name: Option<FirstName>,
    pub gender: Option<Gender>,

    pub bsn: Option<BsnOrNoneConfirmed>,
    pub date_of_birth: Option<Date>,

    pub place_of_residence: Option<PlaceOfResidence>,
    pub country_of_residence: Option<CountryCode>,
}
