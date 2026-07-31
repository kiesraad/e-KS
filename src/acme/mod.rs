//! ACME (Let's Encrypt) certificate renewal via http-01.
//!
//! Each instance renews its own certificate and hot-reloads the server;
//! challenge tokens live in the database so any instance behind the load
//! balancer can answer a validation request. The shared account is deployed
//! as configuration (`ACME_ACCOUNT_CREDENTIALS`), created once with the
//! `create_acme_account` tool. A first boot without cert/key files gets a
//! self-signed placeholder. Apply `deploy/schema.sql` manually before
//! enabling.

mod account;
mod acme_db;
mod acme_store;
mod bootstrap;
mod challenge;
mod renewer;

pub use account::{create_acme_account, parse_acme_account_credentials};
pub(crate) use acme_store::AcmeStore;
pub use bootstrap::bootstrap_certificate;
pub(crate) use challenge::acme_challenge_router;
pub use renewer::run_acme_renewer;
