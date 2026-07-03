use askama::Template;
use axum::{extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};

use crate::{
    AppError, AppState, Context, CsbContext, CsbMainStore, Event, HtmlTemplate, Overlay,
    csb::{CSB_MAIN_STREAM_ID, audit_log::pages::CsbAuditLogDetailPath},
    filters, trans,
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
            description: event.payload.description(locale),
            details: event.payload.details(),
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
            description: event.payload.description(locale),
            details: event.payload.details(),
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
