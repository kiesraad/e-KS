use crate::{
    DutchAddressForm, FullNameForm, TokenValue, substitute_list_submitters::SubstituteSubmitter,
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "SubstituteSubmitter")]
#[serde(default)]
pub struct SubstituteSubmitterForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<SubstituteSubmitter> for SubstituteSubmitterForm {
    fn from(value: SubstituteSubmitter) -> Self {
        SubstituteSubmitterForm {
            name: FullNameForm::from(value.name),
            address: DutchAddressForm::from(value.address),
            csrf_token: Default::default(),
        }
    }
}
