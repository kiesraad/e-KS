use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, audit_log::AuditLogEntry,
    audit_log::pages::AuditLogPath, filters, pagination::Pagination,
};

const PER_PAGE: usize = 20;

#[derive(Template)]
#[template(path = "audit_log/pages/list.html")]
struct AuditLogTemplate {
    entries: Vec<AuditLogEntry>,
    pagination: crate::pagination::PaginationInfo<NoSort>,
}

#[derive(
    Debug, Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct NoSort;

pub async fn audit_log(
    _: AuditLogPath,
    context: Context,
    store: AppStore,
    pagination: Pagination<NoSort>,
) -> Result<impl IntoResponse, AppError> {
    let all_events = store.get_events();
    let total = all_events.len();

    let pagination = Pagination {
        per_page: PER_PAGE,
        ..pagination
    }
    .set_total(total);

    let entries: Vec<AuditLogEntry> = all_events
        .into_iter()
        .rev()
        .skip(pagination.offset())
        .take(pagination.limit())
        .map(Into::into)
        .collect();

    Ok(HtmlTemplate(
        AuditLogTemplate {
            entries,
            pagination,
        },
        context,
    ))
}
