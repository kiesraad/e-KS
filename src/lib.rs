//! Application architecture overview and core types.
//!
//! This application uses event sourcing (see `https://en.wikipedia.org/wiki/Event_sourcing`)
//! with Axum for HTTP routing and Askama for HTML templates.
//!
//! For a more detailed technical overview, see `docs/code-architecture.md`
//! (`https://github.com/kiesraad/e-KS/blob/main/docs/code-architecture.md`).
//!
//! **Persistence configuration**
//! - `STORAGE_URL` selects the persistence backend used by [`PgStore`].
//! - Supported scheme `memory:` disables persistence (in-memory only).
//! - Supported scheme `local://<dir>` stores event streams as files under the provided directory.
//! - Supported scheme `postgres://` or `postgresql://` uses PostgreSQL (requires the `database` feature).
//! - Default (dev) is `postgres://eks@localhost/eks` (see [`Config`]).
//!
//! **Core structs and relationships**
//! - [`AppState`]: application state container shared by request handlers. Owns config,
//!   a `StoreRegistry<PgStoreData>` for per-stream data, and the in-memory
//!   [`SessionStore`] for active sessions.
//! - [`PgStoreData`]: the domain projection for a single stream. It is the
//!   in-memory state updated by [`PgEvent`] through `StoreData::apply`.
//! - `Store<D>`: generic event-sourced store wrapper around type parameter `D`
//!   implementing `StoreData`. It owns
//!   a persistence backend (database/local/memory) and a shared data handle.
//! - [`PgStore`]: type alias for `Store<PgStoreData>`, i.e., the concrete store used
//!   by the application.
//! - `StoreRegistry<D>`: cache/registry that creates and reuses `Store<D>` instances
//!   per stream ID (scoped to BSN + election).
//! - [`PgEvent`]: domain event enum driving updates to [`PgStoreData`].
//!
//! **Event integrity & confidentiality**
//! - Event payloads are encrypted at rest (AES-256-GCM, a random key per stream,
//!   wrapped by a master key) for the file and database backends; see `crypto`.
//! - Persisted events form a hash chain: each event hashes the previous event's
//!   hash plus its own metadata and stored body, making tampering detectable.
//! - Generated PDFs/exports embed the current event ID and chain hash (shown in
//!   the PDF footer), so a document can be tied back to the exact event it was
//!   rendered from.
//!
//! **Directory layout (high level)**
//! - `src/pg/`: political group section domain modules (candidates, candidate_lists,
//!   persons, etc), plus the per-stream [`PgStoreData`] projection (`pg/store/`) and
//!   the HTML error-page renderer.
//! - `src/csb/`: central voting bureau section (import, examination, monitoring).
//! - `src/structs/`: shared domain model structs used by both `pg` and `csb`.
//! - `src/middleware/`: shared HTTP infrastructure that needs [`AppState`]
//!   (session/store middleware, health, proxy, eks_key, dev login).
//! - `src/auth/`: authentication, the session model, and session storage (see [`Session`], [`SessionStore`]).
//! - `src/core/`: shared configuration, logging, server setup, and core helpers (see [`Config`], [`logging`], [`server`]).
//! - `src/store/`: generic event store, persistence, and registry logic (see [`PgStore`]).
//! - `src/state.rs`: [`AppState`] definition and extractors.
//! - `src/router.rs`: top-level route wiring (see [`router`]).
//!
//! **PG module layout (per-domain)**
//! Most `src/pg/<domain>/` modules follow a similar structure:
//! - `pages/`: request handlers, typed paths, and routing glue for HTML flows.
//! - `forms/`: form structs, validation, and submission handling helpers.
//! - `extractors/`: custom request extractors and helper types for handlers.
//! - `structs/`: domain model types used only by this section; shared ones live
//!   in `src/structs/<domain>/` and are re-exported from the domain's `mod.rs`.
//! - `components/`: shared UI/template fragments used across pages.
//! - `mod.rs`: re-exports and module-level wiring.
//!
//! This layout keeps domain-specific routing and UI close to each other while
//! sharing generic infrastructure via `core`, `auth`, `state`, and `store`.

#[cfg(feature = "acme")]
mod acme;
mod auth;
mod core;
mod crypto;
mod csb;
mod error;
mod filters;
mod form;
mod middleware;
mod pg;
mod state;
mod store;
mod structs;
mod utils;

// `pub` because the `pdf_diff` development tool renders the PDF models (from
// the JSON example inputs) through this module.
pub mod models;
pub mod router;

#[cfg(feature = "fixtures")]
mod fixtures;

// The crate's public API is exactly what the binaries need; everything
// else is re-exported `pub(crate)` so the flat `crate::X` import style keeps
// working internally without growing the external interface.
#[cfg(feature = "acme")]
pub use acme::{
    bootstrap_certificate, create_acme_account, parse_acme_account_credentials, run_acme_renewer,
};
pub use auth::session_store::run_session_sweeper;
pub use core::{Config, logging, server};
pub use error::AppError;
pub use state::AppState;
pub use store::run_db_prober;

#[cfg(feature = "acme")]
pub(crate) use acme::AcmeStore;
#[cfg(feature = "acme")]
pub(crate) use core::AcmeConfig;

pub(crate) use middleware::{
    csb_store_middleware, db_gate_middleware, eks_key_middleware, health_router, lb_health_router,
    session_middleware, store_middleware,
};
pub(crate) use pg::{
    Context, PgEvent, PgStore, PgStoreData, audit_log, candidate_lists, candidates, common,
    csrf_rejection_response, finalise, list_designation, list_submitters, name_authorisations,
    persons, political_groups, render_error_pages, substitute_list_submitters,
};

#[cfg(not(feature = "memory-serve"))]
pub(crate) use middleware::proxy_handler;
// `pub` because the `eks` binary's tests reference `eks::SESSION_COOKIE_NAME`.
pub use auth::session_extractor::SESSION_COOKIE_NAME;
pub(crate) use auth::{
    derive_id::IdDeriver, pending_request_store::PendingRequestStore, session::Session,
    session_store::SessionStore,
};
#[cfg(feature = "tls")]
pub(crate) use core::TlsConfig;
pub(crate) use core::{
    AnyLocale, ElectionConfig, ElectionType, ElectoralDistrict, HtmlTemplate, Locale, LocaleValues,
    Province, Scope, SessionPageValues, WaterCouncil,
    constants::{self, MAX_CANDIDATES},
    http_trace, translate,
};
#[cfg(any(test, feature = "dev-features"))]
pub(crate) use csb::CsbMainEvent;
pub(crate) use csb::{CsbContext, CsbEvent, CsbMainStoreData, CsbStoreData};
pub(crate) use error::AppResponse;
pub(crate) use form::{Form, TokenValue};
pub(crate) use state::AppRequestState;
pub(crate) use store::{DbHealth, Event};
pub(crate) use utils::{
    OptionAsStrExt, OptionStringExt, Overlay, QueryParamState, abbreviate_str, id_newtype,
    overlay_active, pagination, redirect_success, success_alert_requested, transparent_string,
};

#[cfg(test)]
pub(crate) use utils::test_utils;

// Nominally `pub` in a private module (then re-exported `pub(crate)`) so the
// many nominally-`pub` signatures mentioning it don't trip `private_interfaces`.
mod stream_id {
    crate::id_newtype!(pub struct StreamId);
}
pub(crate) use stream_id::StreamId;

pub(crate) type CsbStore = store::Store<CsbStoreData>;
pub(crate) type CsbMainStore = store::Store<CsbMainStoreData>;
