use crate::{
    pagination::PaginationInfo,
    persons::{Person, PersonSort},
};

#[derive(Debug, Clone)]
pub struct PersonPagination {
    pub persons: Vec<Person>,
    pub pagination: PaginationInfo<PersonSort>,
}
