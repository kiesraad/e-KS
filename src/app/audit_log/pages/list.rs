use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, audit_log::AuditLogEntry,
    audit_log::pages::AuditLogPath, filters,
};

#[derive(Template)]
#[template(path = "audit_log/pages/list.html")]
struct AuditLogTemplate {
    entries: Vec<AuditLogEntry>,
}

pub async fn audit_log(
    _: AuditLogPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let mut entries: Vec<AuditLogEntry> =
        store.get_events().into_iter().map(Into::into).collect();
    entries.reverse();

    Ok(HtmlTemplate(AuditLogTemplate { entries }, context))
}
