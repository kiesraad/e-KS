use crate::{DutchAddressForm, FullNameForm, TokenValue, list_submitters::ListSubmitter};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "ListSubmitter")]
pub struct ListSubmitterForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<ListSubmitter> for ListSubmitterForm {
    fn from(value: ListSubmitter) -> Self {
        ListSubmitterForm {
            name: FullNameForm::from(value.name),
            address: DutchAddressForm::from(value.address),
            csrf_token: Default::default(),
        }
    }
}
