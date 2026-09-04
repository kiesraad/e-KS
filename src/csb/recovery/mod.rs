//! The "Herstelde lijsten" (recovery) phase: after the omission letters went
//! out, the CSB marks every recoverable omission as recovered or not
//! recovered. Candidates, lists and districts with an unresolved omission are
//! scrapped ("geschrapt").
//!
//! The pages mirror the examination pages under their own route prefix: thin
//! handlers re-render the examination templates in
//! [`CsbPhase::Recovery`](crate::structs::csb::CsbPhase) mode, which hides the
//! examination-only actions and shows the assessment controls instead.
mod pages;
pub(in crate::csb) mod paths;

pub use pages::router;
pub use paths::CsbRecoveryOverviewPath;
