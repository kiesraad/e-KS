use serde::{Deserialize, Serialize};

use crate::form::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviousElectionResults {
    ZeroSeats,
    OneToFifteenSeats,
    SixteenOrMoreSeats,
}

impl std::str::FromStr for PreviousElectionResults {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zero_seats" => Ok(PreviousElectionResults::ZeroSeats),
            "one_to_fifteen_seats" => Ok(PreviousElectionResults::OneToFifteenSeats),
            "sixteen_or_more_seats" => Ok(PreviousElectionResults::SixteenOrMoreSeats),
            _ => Err(ValidationError::InvalidValue),
        }
    }
}

impl std::fmt::Display for PreviousElectionResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreviousElectionResults::ZeroSeats => write!(f, "zero_seats"),
            PreviousElectionResults::OneToFifteenSeats => write!(f, "one_to_fifteen_seats"),
            PreviousElectionResults::SixteenOrMoreSeats => write!(f, "sixteen_or_more_seats"),
        }
    }
}
