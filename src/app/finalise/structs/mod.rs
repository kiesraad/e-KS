//! Data collection for the finalise documents: conversions from the
//! application store types to the PDF model inputs (`crate::models::inputs`)
//! and the EML export.

mod candidate;
mod detailed_candidate;
pub mod documents;
mod electoral_districts;
mod eml210;
mod h1;
mod h3;
mod h4;
mod h9;
mod name_authorisation;
mod person;
mod postal_address;
pub mod problems;
mod problems_sort;

use candidate::ordered_candidates;
