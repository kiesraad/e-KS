//! Application architecture overview and core types.
//!
//! This application uses event sourcing (see `https://en.wikipedia.org/wiki/Event_sourcing`)
//! with Axum for HTTP routing and Askama for HTML templates.
//!
//! For a more detailed technical overview, see `docs/code-architecture.md`
//! (`https://github.com/kiesraad/e-KS/blob/main/docs/code-architecture.md`).
//!
//! **Persistence configuration**
//! - `STORAGE_URL` selects the persistence backend used by [`AppStore`].
//! - Supported scheme `memory:` disables persistence (in-memory only).
//! - Supported scheme `local://<dir>` stores event streams as files under the provided directory.
//! - Supported scheme `postgres://` or `postgresql://` uses PostgreSQL (requires the `database` feature).
//! - Default (dev) is `postgres://eks@localhost/eks` (see [`Config`]).
//!
//! **Core structs and relationships**
//! - [`AppState`]: application state container shared by request handlers. Owns config,
//!   a `StoreRegistry<AppStoreData>` for per-stream data, and the in-memory
//!   [`SessionStore`] for active sessions.
//! - [`AppStoreData`]: the domain projection for a single stream. It is the
//!   in-memory state updated by [`AppEvent`] through `StoreData::apply`.
//! - `Store<D>`: generic event-sourced store wrapper around type parameter `D`
//!   implementing `StoreData`. It owns
//!   a persistence backend (database/local/memory) and a shared data handle.
//! - [`AppStore`]: type alias for `Store<AppStoreData>`, i.e., the concrete store used
//!   by the application.
//! - `StoreRegistry<D>`: cache/registry that creates and reuses `Store<D>` instances
//!   per stream ID (scoped to BSN + election).
//! - [`AppEvent`]: domain event enum driving updates to [`AppStoreData`].
//!
//! **Event integrity & confidentiality**
//! - Event payloads are encrypted at rest (AES-256-GCM, per-stream keys) for the
//!   file and database backends; see `store::EventEncryption`.
//! - Persisted events form a hash chain: each event hashes the previous event's
//!   hash plus its own metadata and stored body, making tampering detectable.
//! - Generated PDFs/exports embed the current event ID and chain hash (shown in
//!   the PDF footer), so a document can be tied back to the exact event it was
//!   rendered from.
//!
//! **Directory layout (high level)**
//! - `src/app/`: application domain modules (candidates, candidate_lists, persons, etc),
//!   plus the per-stream [`AppStoreData`] projection (`app/store/`), shared HTTP
//!   infrastructure that needs [`AppState`] (`app/middleware/`: health, proxy,
//!   eks_key), and the HTML error-page renderer.
//! - `src/auth/`: authentication, sessions, and session extractors (see [`Session`], [`SessionStore`]).
//! - `src/core/`: shared configuration, logging, server setup, and core helpers (see [`Config`], [`logging`], [`server`]).
//! - `src/store/`: generic event store, persistence, and registry logic (see [`AppStore`]).
//! - `src/state.rs`: [`AppState`] definition and extractors.
//! - `src/router.rs`: top-level route wiring (see [`router`]).
//!
//! **App module layout (per-domain)**
//! Most `src/app/<domain>/` modules follow a similar structure:
//! - `pages/`: request handlers, typed paths, and routing glue for HTML flows.
//! - `forms/`: form structs, validation, and submission handling helpers.
//! - `extractors/`: custom request extractors and helper types for handlers.
//! - `structs/`: domain model types used by pages and store projections.
//! - `components/`: shared UI/template fragments used across pages.
//! - `mod.rs`: re-exports and module-level wiring.
//!
//! This layout keeps domain-specific routing and UI close to each other while
//! sharing generic infrastructure via `core`, `auth`, `state`, and `store`.

mod app;
mod auth;
mod core;
pub mod csb;
mod error;
mod form;
mod pagination;
mod state;
mod store;

pub mod filters;
pub mod router;
pub mod utils;

#[cfg(feature = "fixtures")]
pub mod fixtures;

pub use app::{
    AppEvent, AppStoreData, Context, ErrorResponse, audit_log, candidate_lists, candidates, common,
    db_gate_middleware, eks_key_middleware, finalise, handle_db_error, health_router,
    list_designation, list_submitters, name_authorisations, persons, political_groups,
    render_error_pages, substitute_list_submitters,
};

#[cfg(any(feature = "dev-features", not(feature = "memory-serve")))]
pub use app::proxy_handler;
pub use auth::{
    derive_id::IdDeriver,
    pending_request_store::PendingRequestStore,
    scope::Scope,
    session::{Session, session_idle_timeout},
    session_extractor::{
        SESSION_COOKIE_NAME, csb_store_middleware, session_middleware, store_middleware,
    },
    session_store::{SessionStore, run_session_sweeper},
};
pub use core::{
    AnyLocale, Config, ElectionConfig, ElectionType, ElectoralDistrict, HtmlTemplate, Locale,
    Province, TlsConfig, TypstRenderer, WaterCouncil,
    constants::{self, MAX_CANDIDATES},
    get_env, http_trace, logging, server, translate,
};
pub use csb::{CsbContext, CsbEvent, CsbMainEvent, CsbMainStoreData, CsbStoreData};
pub use error::{AppError, AppResponse};
pub use form::{Form, TokenValue};
pub use state::{AppRequestState, AppState};
pub use store::{DbHealth, Event, HealthState, run_db_prober};
pub use utils::{
    OptionAsStrExt, OptionStringExt, Overlay, QueryParamState, abbreviate_str, id_newtype,
    redirect_success, transparent_string,
};

#[cfg(test)]
pub use utils::test_utils;

id_newtype!(pub struct StreamId);

pub type AppStore = store::Store<AppStoreData>;

pub type CsbStore = store::Store<CsbStoreData>;
pub type CsbMainStore = store::Store<CsbMainStoreData>;
