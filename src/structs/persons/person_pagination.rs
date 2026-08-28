use super::PersonWithProblems;
use crate::{pagination::PaginationInfo, structs::persons::PersonSort};

#[derive(Debug, Clone)]
pub struct PersonPagination {
    pub persons: Vec<PersonWithProblems>,
    pub pagination: PaginationInfo<PersonSort>,
}
