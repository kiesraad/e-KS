use crate::form::ValidationError;

#[derive(Default, Debug, Clone)]
pub enum FormAction {
    #[default]
    Save,
    Remove,
}

impl std::str::FromStr for FormAction {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "save" => Ok(FormAction::Save),
            "remove" => Ok(FormAction::Remove),
            _ => Err(ValidationError::InvalidValue),
        }
    }
}

impl std::fmt::Display for FormAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormAction::Save => write!(f, "save"),
            FormAction::Remove => write!(f, "remove"),
        }
    }
}
