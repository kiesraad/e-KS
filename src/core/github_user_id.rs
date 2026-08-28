//! Constrained newtype for the numeric id of a GitHub account.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Numeric GitHub account id. The allowlist for the CSB login is expressed in
/// these ids rather than login names: an id is stable for the lifetime of an
/// account, while a released login name can be re-registered by someone else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct GithubUserId(u64);

impl TryFrom<u64> for GithubUserId {
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err("GitHub user id must be a positive integer".to_string());
        }
        Ok(Self(value))
    }
}

impl From<GithubUserId> for u64 {
    fn from(id: GithubUserId) -> Self {
        id.0
    }
}

impl FromStr for GithubUserId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u64>()
            .map_err(|_| format!("invalid GitHub user id: {value:?}"))?
            .try_into()
    }
}

impl Display for GithubUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_positive_integers() {
        let id: GithubUserId = "583231".parse().expect("valid id");
        assert_eq!(id.to_string(), "583231");
        assert_eq!(u64::from(id), 583231);
    }

    #[test]
    fn rejects_zero_and_non_numeric_values() {
        assert!("0".parse::<GithubUserId>().is_err());
        assert!("".parse::<GithubUserId>().is_err());
        assert!("-1".parse::<GithubUserId>().is_err());
        assert!("octocat".parse::<GithubUserId>().is_err());
    }

    #[test]
    fn serde_roundtrips_as_plain_number() {
        let id: GithubUserId = "42".parse().expect("valid id");
        let json = serde_json::to_string(&id).expect("serialize");
        assert_eq!(json, "42");
        let back: GithubUserId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, id);
        assert!(serde_json::from_str::<GithubUserId>("0").is_err());
    }
}
