mod candidate_pages;
mod candidate_repository;
mod extractors;
mod pages;
mod repository;
mod structs;

pub use candidate_pages::candidate_router;
pub use candidate_repository::*;
pub use pages::router;
pub use repository::*;
pub use structs::*;
