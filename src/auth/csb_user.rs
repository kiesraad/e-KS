//! The authenticated identity behind a CSB (committee) session.

use serde::{Deserialize, Serialize};

use crate::{AppError, GithubUserId, Locale, Session, StreamId, trans, utils::abbreviate_str};

/// The committee member behind a CSB session, recorded on every CSB event so
/// the audit log can show who triggered it.
///
/// Deliberately an enum over login methods rather than a single id: future
/// login methods add a variant here, and the events referencing the user
/// stay unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CsbUser {
    /// Dev-login bypass, identified only by the session's derived stream id.
    Developer { stream_id: StreamId },
    /// GitHub OAuth login, identified by the account's numeric id.
    Github { user_id: GithubUserId },
}

/// Implemented by the CSB store events, which all record the committee member
/// that triggered them. Lets the audit log render the user generically.
pub trait HasCsbUser {
    fn csb_user(&self) -> &CsbUser;
}

impl CsbUser {
    /// Human-readable label shown in the audit log.
    pub fn describe(&self, locale: Locale) -> String {
        match self {
            CsbUser::Developer { stream_id } => format!(
                "{} {}",
                trans!("audit_log.user.developer", locale),
                abbreviate_str(&stream_id.to_string())
            ),
            CsbUser::Github { user_id } => {
                format!("{} {user_id}", trans!("audit_log.user.github", locale))
            }
        }
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        CsbUser::Developer {
            stream_id: StreamId::new(),
        }
    }
}

impl Session {
    /// The committee identity of this session, or `Unauthorised` when the
    /// session was not established through a CSB login (or predates one).
    pub fn require_csb_user(&self) -> Result<CsbUser, AppError> {
        self.csb_user.clone().ok_or(AppError::Unauthorised)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_shows_login_method_and_identity() {
        let github = CsbUser::Github {
            user_id: "583231".parse().expect("valid id"),
        };
        assert_eq!(github.describe(Locale::En), "GitHub user 583231");
        assert_eq!(github.describe(Locale::Nl), "GitHub-gebruiker 583231");

        let stream_id = StreamId::new();
        let developer = CsbUser::Developer { stream_id };
        let label = developer.describe(Locale::En);
        assert!(label.starts_with("Developer "));
        assert!(label.contains(&abbreviate_str(&stream_id.to_string())));
    }

    #[test]
    fn serde_roundtrips() {
        let user = CsbUser::Github {
            user_id: "42".parse().expect("valid id"),
        };
        let json = serde_json::to_string(&user).expect("serialize");
        let back: CsbUser = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, user);
    }
}
