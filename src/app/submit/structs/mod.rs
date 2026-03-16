pub mod h1;
pub mod h3_1;
pub mod h9;
mod typst_authorised_agent;
pub mod typst_candidate;
mod typst_datetime;
mod typst_detailed_candidate;
pub mod typst_electoral_districts;
mod typst_person;
mod typst_postal_address;

use typst_candidate::{TypstCandidate, ordered_candidates};
use typst_datetime::TypstDatetime;
use typst_electoral_districts::TypstElectoralDistricts;
use typst_person::{TypstPerson, substitute_submitter_from_ids};
