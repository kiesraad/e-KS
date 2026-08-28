use serde::{Deserialize, Serialize};

use crate::{CsbUser, Event, HasCsbUser, trans};

/// An event on the global CSB stream: the acting committee member plus what
/// they did. Every event records its user so the audit log can show who
/// triggered it.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CsbMainEvent {
    /// The committee member that triggered the event.
    pub user: CsbUser,
    pub action: CsbMainAction,
}

/// Actions on the global CSB stream. Variants will be added as committee-wide
/// features are implemented (process steps, audit log, etc.).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CsbMainAction {
    /// A committee member logged in; the login method is carried by the
    /// event's [`CsbUser`].
    Login,
}

impl CsbMainAction {
    /// Attach the acting committee member, producing the event to persist.
    pub fn by(self, user: CsbUser) -> CsbMainEvent {
        CsbMainEvent { user, action: self }
    }
}

impl HasCsbUser for CsbMainEvent {
    fn csb_user(&self) -> &CsbUser {
        &self.user
    }
}

impl Event for CsbMainEvent {
    fn category(&self) -> &'static str {
        match self.action {
            CsbMainAction::Login => "system",
        }
    }

    fn key(&self) -> &'static str {
        match self.action {
            CsbMainAction::Login => "login",
        }
    }

    fn description(&self, locale: crate::Locale) -> String {
        match self.action {
            CsbMainAction::Login => trans!("audit_log.event.login", locale),
        }
    }

    fn details(&self) -> String {
        match self.action {
            CsbMainAction::Login => String::new(),
        }
    }
}
