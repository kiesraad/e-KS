//! Application architecture overview and core types.
//!
//! This application uses event sourcing (see `https://en.wikipedia.org/wiki/Event_sourcing`)
//! with Axum for HTTP routing and Askama for HTML templates.
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
//!   a `StoreRegistry<AppStoreData>` for per-political-group data, and the in-memory
//!   [`SessionStore`] for active sessions.
//! - [`AppStoreData`]: the domain projection for a single political group. It is the
//!   in-memory state updated by [`AppEvent`] through `StoreData::apply`.
//! - `Store<D>`: generic event-sourced store wrapper around type parameter `D`
//!   implementing `StoreData`. It owns
//!   a persistence backend (database/local/memory) and a shared data handle.
//! - [`AppStore`]: type alias for `Store<AppStoreData>`, i.e., the concrete store used
//!   by the application.
//! - `StoreRegistry<D>`: cache/registry that creates and reuses `Store<D>` instances
//!   per stream ID (political group).
//! - [`AppEvent`]: domain event enum driving updates to [`AppStoreData`].
//!
//! **Directory layout (high level)**
//! - `src/app/`: application domain modules (candidates, candidate_lists, persons, etc).
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
    AppEvent, AppStoreData, Context, authorised_agents, candidate_lists, candidates, common,
    list_submitters, persons, political_groups, submit, substitute_list_submitters,
};
pub use auth::{
    session::{SESSION_IDLE_TIMEOUT, Session},
    session_extractor::{SESSION_COOKIE_NAME, session_middleware, store_middleware},
    session_store::SessionStore,
};
pub use core::{
    Config, ElectionConfig, ElectoralDistrict, HtmlTemplate, Locale, constants, get_env, logging,
    server, translate,
};
pub use error::{AppError, AppResponse, ErrorResponse, render_error_pages};
pub use form::{CsrfToken, CsrfTokens, Form, TokenValue};
pub use state::AppState;
pub use utils::{OptionStringExt, QueryParamState, new_type, redirect_success};

#[cfg(test)]
pub use utils::test_utils;

id_newtype!(pub struct PoliticalGroupId);

pub type AppStore = store::Store<AppStoreData>;
