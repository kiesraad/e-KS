/// The CSB process phase a page is rendered for. The examination pages double
/// as the "Herstelde lijsten" (recovery) pages: the same templates render in
/// either mode, with the mutating actions (paper corrections, corrections,
/// adding omissions) reserved for the examination phase and the
/// recovered / not-recovered assessment reserved for the recovery phase.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CsbPhase {
    #[default]
    Examination,
    Recovery,
}

impl CsbPhase {
    pub fn is_examination(&self) -> bool {
        matches!(self, CsbPhase::Examination)
    }

    pub fn is_recovery(&self) -> bool {
        matches!(self, CsbPhase::Recovery)
    }
}
