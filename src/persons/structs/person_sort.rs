use serde::{Deserialize, Serialize};
use strum::AsRefStr;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, AsRefStr, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PersonSort {
    #[default]
    LastName,
    FirstName,
    Initials,
    Gender,
    Locality,
    CreatedAt,
    UpdatedAt,
}
