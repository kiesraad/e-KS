use derive_more::{Display, FromStr};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
#[display(rename_all = "snake_case")]
#[from_str(rename_all = "snake_case")]
pub enum FormAction {
    #[default]
    Save,
    Remove,
}
