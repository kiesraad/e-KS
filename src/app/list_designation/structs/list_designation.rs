use crate::{
    common::{LegalName, PreviousElectionResults},
    form::ValidationError,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalEntity {
    pub previous_election_results: Option<PreviousElectionResults>,
    pub legal_name: Option<LegalName>,
}

impl PoliticalEntity {
    pub fn get_max_candidates(&self) -> usize {
        match self.previous_election_results {
            Some(PreviousElectionResults::SixteenOrMoreSeats) => 80,
            _ => 50,
        }
    }

    pub fn was_previously_seated(&self) -> bool {
        self.previous_election_results
            .is_some_and(|r| r != PreviousElectionResults::ZeroSeats)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListDesignation {
    Standalone(PoliticalEntity),
    Blank,
    Combined(Vec<PoliticalEntity>),
}

impl ListDesignation {
    pub fn get_max_candidates(&self) -> usize {
        match self {
            ListDesignation::Standalone(entity) => entity.get_max_candidates(),
            ListDesignation::Combined(entities) => entities
                .iter()
                .map(|g| g.get_max_candidates())
                .max()
                .unwrap_or(50),
            ListDesignation::Blank => 50,
        }
    }

    pub fn was_previously_seated(&self) -> bool {
        match self {
            ListDesignation::Standalone(entity) => entity.was_previously_seated(),
            ListDesignation::Blank => false,
            ListDesignation::Combined(entities) => {
                entities.iter().any(|e| e.was_previously_seated())
            }
        }
    }
}

impl Default for ListDesignation {
    fn default() -> Self {
        ListDesignation::Standalone(Default::default())
    }
}

impl std::str::FromStr for ListDesignation {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standalone" => Ok(ListDesignation::Standalone(Default::default())),
            "blank" => Ok(ListDesignation::Blank),
            "combined" => Ok(ListDesignation::Combined(Default::default())),
            _ => Err(ValidationError::InvalidValue),
        }
    }
}

impl std::fmt::Display for ListDesignation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListDesignation::Standalone(_) => write!(f, "standalone"),
            ListDesignation::Blank => write!(f, "blank"),
            ListDesignation::Combined { .. } => write!(f, "combined"),
        }
    }
}
