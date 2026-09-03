use std::fmt::Display;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::structs::common::UtcDateTime;

/// How long an `InProgress` sweep may go without finishing before it is
/// treated as abandoned. A sweep only lives inside one process, so a restart
/// halfway through would otherwise leave the stream `InProgress` forever, with
/// the re-entrancy guard refusing to ever start another one.
pub const BRP_SWEEP_STALE_AFTER: chrono::TimeDelta = chrono::TimeDelta::hours(1);

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum BrpStatus {
    #[default]
    NotStarted,
    /// A sweep is running. `started_at` is what makes an abandoned sweep
    /// recognisable; see [`BRP_SWEEP_STALE_AFTER`].
    InProgress {
        started_at: UtcDateTime,
    },
    /// The sweep stopped early, for instance because the BRP was unreachable.
    /// Candidates checked before that point keep their findings; the rest were
    /// not checked at all.
    Aborted(String),
    Finished,
}

impl BrpStatus {
    /// Whether a sweep is running and still recent enough to wait for.
    ///
    /// An `InProgress` status older than [`BRP_SWEEP_STALE_AFTER`] belongs to a
    /// process that is gone, so it does not block a new sweep.
    pub fn is_running(&self) -> bool {
        match self {
            Self::InProgress { started_at } => {
                Utc::now().signed_duration_since(**started_at) < BRP_SWEEP_STALE_AFTER
            }
            _ => false,
        }
    }

    /// A sweep that starts now.
    pub fn in_progress() -> Self {
        Self::InProgress {
            started_at: UtcDateTime::now(),
        }
    }
}

impl Display for BrpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrpStatus::NotStarted => write!(f, "not_started"),
            BrpStatus::InProgress { .. } => write!(f, "in_progress"),
            BrpStatus::Aborted(_) => write!(f, "aborted"),
            BrpStatus::Finished => write!(f, "finished"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_sweep_is_running() {
        assert!(BrpStatus::in_progress().is_running());
    }

    #[test]
    fn an_abandoned_sweep_does_not_block_a_new_one() {
        let stale = BrpStatus::InProgress {
            started_at: (Utc::now() - BRP_SWEEP_STALE_AFTER - chrono::TimeDelta::seconds(1)).into(),
        };

        assert!(!stale.is_running());
    }

    #[test]
    fn no_other_status_counts_as_running() {
        for status in [
            BrpStatus::NotStarted,
            BrpStatus::Finished,
            BrpStatus::Aborted("upstream unreachable".to_string()),
        ] {
            assert!(!status.is_running(), "{status} should not count as running");
        }
    }
}
