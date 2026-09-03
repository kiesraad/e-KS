use crate::{
    Locale,
    structs::brp::{BrpCheckedField, BrpFinding, BrpStatus},
    trans,
};

/// The BRP findings for one candidate, grouped the way the candidate detail
/// table is laid out and with every message already translated, so the
/// template only has to render lists of strings.
#[derive(Debug, Default)]
pub struct CandidateBrpFindings {
    pub bsn: Vec<String>,
    pub initials: Vec<String>,
    pub last_name: Vec<String>,
    pub gender: Vec<String>,
    pub date_of_birth: Vec<String>,
    pub place_of_residence: Vec<String>,
    /// Findings about the candidate as a whole rather than about one row of
    /// the table: whether they can be elected at all.
    pub candidate: Vec<String>,
}

impl CandidateBrpFindings {
    pub fn new(findings: &[BrpFinding], locale: Locale) -> Self {
        let mut grouped = Self::default();

        for finding in findings {
            let message = finding.message(locale);
            let target = match finding.field() {
                Some(BrpCheckedField::Bsn) => &mut grouped.bsn,
                Some(BrpCheckedField::Initials) => &mut grouped.initials,
                Some(BrpCheckedField::LastName) => &mut grouped.last_name,
                Some(BrpCheckedField::Gender) => &mut grouped.gender,
                Some(BrpCheckedField::DateOfBirth) => &mut grouped.date_of_birth,
                Some(BrpCheckedField::PlaceOfResidence) => &mut grouped.place_of_residence,
                None => &mut grouped.candidate,
            };
            target.push(message);
        }

        grouped
    }
}

/// Why the BRP data on screen may be incomplete, or `None` when the check ran
/// to completion.
///
/// A check that stopped early leaves candidates unchecked, and an empty findings
/// list would otherwise be indistinguishable from "the BRP agreed on
/// everything" -- so the committee is told which of the two it is looking at.
pub fn brp_incomplete_reason(status: &BrpStatus, locale: Locale) -> Option<String> {
    match status {
        BrpStatus::Finished => None,
        BrpStatus::NotStarted => Some(trans!("csb.brp.status.not_started", locale)),
        BrpStatus::InProgress { .. } => Some(trans!("csb.brp.status.in_progress", locale)),
        BrpStatus::Aborted(error) => Some(trans!("csb.brp.status.aborted", locale, error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn findings_are_grouped_onto_the_row_they_belong_to() {
        let grouped = CandidateBrpFindings::new(
            &[
                BrpFinding::Mismatch {
                    field: BrpCheckedField::LastName,
                    brp_value: "de Bruin".to_string(),
                },
                BrpFinding::ResidenceAbroad,
                BrpFinding::NotDutch,
            ],
            Locale::En,
        );

        assert_eq!(grouped.last_name.len(), 1);
        // A residence finding belongs on the place-of-residence row.
        assert_eq!(grouped.place_of_residence.len(), 1);
        // Nationality is about the candidate, not about a row.
        assert_eq!(grouped.candidate.len(), 1);
        assert!(grouped.initials.is_empty());
    }

    #[test]
    fn only_an_incomplete_check_is_reported() {
        assert!(brp_incomplete_reason(&BrpStatus::Finished, Locale::En).is_none());
        assert!(brp_incomplete_reason(&BrpStatus::NotStarted, Locale::En).is_some());
        assert!(brp_incomplete_reason(&BrpStatus::in_progress(), Locale::En).is_some());

        let aborted = BrpStatus::Aborted("upstream unreachable".to_string());
        let reason = brp_incomplete_reason(&aborted, Locale::En).expect("aborted needs a reason");
        assert!(
            reason.contains("upstream unreachable"),
            "the reason should name what went wrong, got: {reason}"
        );
    }
}
