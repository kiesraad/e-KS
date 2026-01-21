use crate::{
    pagination::PaginationInfo,
    persons::{Person, PersonSort},
};

#[derive(Debug, Clone)]
pub struct PersonPagination {
    pub persons: Vec<Person>,
    pub pagination: PaginationInfo<PersonSort>,
}

impl PersonPagination {
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            persons: Vec::new(),
            pagination: crate::pagination::Pagination::default().set_total(0),
        }
    }
}
