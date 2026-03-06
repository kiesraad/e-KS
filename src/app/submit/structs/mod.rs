pub mod electoral_districts;
pub mod h1;
pub mod h9;
pub mod typst_candidate;
mod typst_datetime;
mod typst_detailed_candidate;
mod typst_person;
mod typst_postal_address;

use electoral_districts::ElectoralDistricts;
use typst_candidate::{TypstCandidate, ordered_candidates};
use typst_datetime::TypstDatetime;
use typst_person::{TypstPerson, substitute_submitter_from_ids};
