use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, FromStr,
)]
#[serde(rename_all = "lowercase")]
#[display(rename_all = "lowercase")]
#[from_str(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
}
