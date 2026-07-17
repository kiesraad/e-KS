//! Request-scoped template context carrying locale and helpers.
//! Extracted from requests and passed into Askama templates.

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::routing::TypedPath;

use crate::{AppError, AppRequestState, AppStore, ElectionConfig, Session};

#[cfg(test)]
use crate::Locale;

/// Request-scoped template context used by Askama.
#[derive(Clone)]
pub struct Context {
    /// Election configuration for this stream.
    pub election: ElectionConfig,
    /// Maximum number of candidates allowed for this political group.
    pub max_candidates: usize,
    /// Multiple candidate lists present
    pub multiple_candidate_lists: bool,
    /// Whether to show the success alert based on the request query.
    pub show_success_alert: bool,
    /// Whether to show a warning that documents were downloaded and changes won't be reflected.
    pub show_download_warning: bool,
    /// Whether the request came from an overlay page (via referrer query).
    pub overlay_referrer: bool,
    /// Session data for locale and CSRF.
    pub session: Session,
    /// Short identifier of the server this instance runs on (e.g. "S1"),
    /// rendered next to the version in the layout footer when set.
    pub server_name: Option<&'static str>,
    /// URL for the "General information" nav link. Includes `initial=true` when
    /// general information is still empty, so the first-visit flow suppresses warnings.
    pub general_information_path: String,
    /// Whether a CSB session is correcting a stream's paper documents.
    pub paper_correction_mode: bool,
    /// URL that leaves paper-corrections mode, set only in paper-corrections
    /// mode. Empty otherwise.
    pub paper_corrections_exit_path: String,
    /// Display name of the political group being corrected, shown in the
    /// corrections banner. Empty outside paper-corrections mode.
    pub paper_corrections_group_name: String,
}

impl Context {
    pub fn new(store: &AppStore, session: Session) -> Self {
        let election = store.get_election();
        let political_group = store.get_political_group();
        let max_candidates = political_group.get_max_candidates();
        let multiple_candidate_lists = store.get_candidate_list_count() > 1;

        let general_information_path = political_group.general_information_path(store);

        let paper_correction_mode = store.paper_corrections_stream_id().is_some();

        let paper_corrections_exit_path = store
            .paper_corrections_stream_id()
            .map(|stream_id| {
                crate::csb::examination::CsbPaperCorrectionsStopPath { stream_id }.to_string()
            })
            .unwrap_or_default();

        let paper_corrections_group_name = if paper_correction_mode {
            political_group.csb_display_name(store.first_candidate_name().as_ref())
        } else {
            String::new()
        };

        Self {
            election,
            max_candidates,
            multiple_candidate_lists,
            show_success_alert: false,
            show_download_warning: false,
            overlay_referrer: false,
            session,
            server_name: None,
            general_information_path,
            paper_correction_mode,
            paper_corrections_exit_path,
            paper_corrections_group_name,
        }
    }

    #[cfg(test)]
    pub fn new_test_without_db() -> Self {
        let store = AppStore::new_for_test();
        Self::new(&store, Session::new_test_with_locale(Locale::En))
    }

    #[cfg(test)]
    pub fn new_test_from_store(store: &AppStore) -> Self {
        Self::new(store, Session::new_test_with_locale(Locale::En))
    }

    pub fn livereload_enabled() -> bool {
        cfg!(feature = "livereload")
    }
}

impl askama::Values for Context {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.session.locale as &dyn std::any::Any),
            "csrf_token" => Some(&self.session.csrf_token().0 as &dyn std::any::Any),
            "election" => Some(&self.election as &dyn std::any::Any),
            "max_candidates" => Some(&self.max_candidates as &dyn std::any::Any),
            "show_success_alert" => Some(&self.show_success_alert as &dyn std::any::Any),
            "show_download_warning" => Some(&self.show_download_warning as &dyn std::any::Any),
            "multiple_candidate_lists" => {
                Some(&self.multiple_candidate_lists as &dyn std::any::Any)
            }
            "overlay_referrer" => Some(&self.overlay_referrer as &dyn std::any::Any),
            "server_name" => Some(&self.server_name as &dyn std::any::Any),
            "general_information_path" => {
                Some(&self.general_information_path as &dyn std::any::Any)
            }
            "paper_correction_mode" => Some(&self.paper_correction_mode as &dyn std::any::Any),
            "paper_corrections_exit_path" => {
                Some(&self.paper_corrections_exit_path as &dyn std::any::Any)
            }
            "paper_corrections_group_name" => {
                Some(&self.paper_corrections_group_name as &dyn std::any::Any)
            }
            _ => None,
        }
    }
}

impl<S: AppRequestState> FromRequestParts<S> for Context {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let store = AppStore::from_request_parts(parts, state).await?;
        let mut context = Context::new(&store, session);

        context.server_name = state.config().server_name.as_deref();

        let path = parts.uri.path();
        context.show_download_warning = store.should_show_download_warning()
            && (path.starts_with(crate::list_designation::ListDesignationUpdatePath::PATH)
                || path.starts_with(crate::candidate_lists::CandidateListsPath::PATH)
                || path.starts_with(crate::persons::PersonsPath::PATH));

        context.show_success_alert = crate::success_alert_requested(parts);
        context.overlay_referrer = crate::overlay_referrer(parts);

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_context_sets_locale() {
        let context = Context::new_test_without_db();
        assert_eq!(context.session.locale, Locale::En);
    }

    #[test]
    fn livereload_flag_matches_feature() {
        assert_eq!(Context::livereload_enabled(), cfg!(feature = "livereload"));
    }
}
