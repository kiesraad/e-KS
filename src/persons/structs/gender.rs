use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Display, EnumString,
)]
#[sqlx(type_name = "gender", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    X,
}
