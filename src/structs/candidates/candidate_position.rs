use crate::common::FormAction;

#[derive(Debug, Default, Clone)]
pub struct CandidatePosition {
    pub position: usize,
    pub action: FormAction,
}
