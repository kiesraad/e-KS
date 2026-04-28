use std::str::FromStr;

use crate::{
    OptionAsStrExt,
    form::{ValidationError, validate_length, validate_teletex_chars},
    submit::{Completable, IncompleteItem},
    transparent_string,
};

pub type FirstName = ConstrainedString;
pub type LegalName = ConstrainedString;
pub type StreetName = ConstrainedString;
pub type Locality = ConstrainedString;
pub type PlaceOfResidence = ConstrainedString;
pub type StateOrProvince = ConstrainedString;

transparent_string! {
    pub struct ConstrainedString(String);
}

impl FromStr for ConstrainedString {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = validate_length(value, 2, 200)?;
        validate_teletex_chars(&trimmed_value)?;

        Ok(ConstrainedString(trimmed_value))
    }
}

impl Completable for Option<LegalName> {
    fn incomplete_items(&self) -> Vec<IncompleteItem> {
        if self.is_empty_or_none() {
            vec![IncompleteItem::NoLegalName]
        } else {
            Vec::new()
        }
    }
}
