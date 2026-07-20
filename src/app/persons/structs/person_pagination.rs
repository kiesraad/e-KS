use crate::{
    pagination::PaginationInfo,
    persons::{PersonSort, structs::PersonWithProblems},
};

#[derive(Debug, Clone)]
pub struct PersonPagination {
    pub persons: Vec<PersonWithProblems>,
    pub pagination: PaginationInfo<PersonSort>,
}
