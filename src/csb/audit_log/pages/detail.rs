use askama::Template;
use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};

use crate::{
    AppError, AppState, Context, CsbContext, CsbEvent, CsbMainEvent, CsbMainStore, HtmlTemplate,
    Locale, Overlay,
    csb::{CSB_MAIN_STREAM_ID, audit_log::pages::CsbAuditLogDetailPath},
    filters, trans,
    utils::format_hash,
};

struct CsbEventDetail {
    event_id: usize,
    stream_label: String,
    description: String,
    details: String,
    created_at: DateTime<Utc>,
}

#[derive(Template)]
#[template(path = "csb/audit_log/pages/detail.html")]
struct CsbAuditLogDetailTemplate {
    detail: CsbEventDetail,
    overlay: Overlay,
}

pub async fn csb_audit_log_detail(
    CsbAuditLogDetailPath {
        stream_id,
        event_id,
    }: CsbAuditLogDetailPath,
    context: CsbContext,
    main_store: CsbMainStore,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let locale = context.session.locale;
    let detail = if stream_id == CSB_MAIN_STREAM_ID {
        let data = main_store.data.read();
        let event = data
            .events
            .iter()
            .find(|e| e.event_id == event_id)
            .ok_or(AppError::GenericNotFound)?;
        CsbEventDetail {
            event_id: event.event_id,
            stream_label: trans!("audit_log.filter.csb_main_stream", locale),
            description: main_event_description(&event.payload, locale),
            details: String::new(),
            created_at: event.created_at,
        }
    } else {
        let import_stores = state.csb_store_registry.stores_by_scope().await?;
        let store = import_stores
            .iter()
            .find(|s| s.stream_id == stream_id)
            .ok_or(AppError::GenericNotFound)?;
        let data = store.data.read();

        let event = data
            .events
            .iter()
            .find(|e| e.event_id == event_id)
            .ok_or(AppError::GenericNotFound)?;
        CsbEventDetail {
            event_id: event.event_id,
            stream_label: store.get_political_group().csb_display_name(),
            description: import_event_description(&event.payload, locale),
            details: import_event_details(&event.payload),
            created_at: event.created_at,
        }
    };

    Ok(HtmlTemplate(
        CsbAuditLogDetailTemplate {
            detail,
            overlay: Overlay::default(),
        },
        context,
    ))
}

fn main_event_description(event: &CsbMainEvent, locale: Locale) -> String {
    match event {
        CsbMainEvent::DeveloperLogin { .. } => trans!("audit_log.event.developer_login", locale),
    }
}

fn import_event_description(event: &CsbEvent, locale: Locale) -> String {
    match event {
        CsbEvent::Import { .. } => trans!("audit_log.event.import", locale),
        CsbEvent::SetFinished(_) => trans!("audit_log.event.set_finished", locale),
        CsbEvent::CreateOmission(_) => trans!("audit_log.event.create_omission", locale),
        CsbEvent::UpdateOmission(_) => trans!("audit_log.event.update_omission", locale),
        CsbEvent::DeleteOmission { .. } => trans!("audit_log.event.delete_omission", locale),
    }
}

fn import_event_details(event: &CsbEvent) -> String {
    match event {
        CsbEvent::Import {
            hash,
            source_stream_id,
            ..
        } => {
            format!(
                "Hash: {}\nSource stream: {source_stream_id}",
                format_hash(hash, true)
            )
        }
        CsbEvent::SetFinished(value) => value.to_string(),
        CsbEvent::CreateOmission(o) | CsbEvent::UpdateOmission(o) => o.description.clone(),
        CsbEvent::DeleteOmission { omission_id } => omission_id.to_string(),
    }
}
