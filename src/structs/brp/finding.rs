use serde::{Deserialize, Serialize};

use crate::{Locale, trans};

/// The candidate fields that are compared against the BRP: the data that is
/// printed on the candidate list (model H 1), plus the date of birth and the
/// burgerservicenummer.
///
/// The correspondence address is deliberately absent. It is not published on
/// the list, and the BRP models an address differently than this application
/// does (a separate `huisletter`, a street name truncated to 24 characters),
/// so comparing the two produces differences that mean nothing to the
/// committee. Only the `woonplaats` is verified, through
/// [`Self::PlaceOfResidence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrpCheckedField {
    Bsn,
    Initials,
    /// Last name including its prefix, matching how the candidate detail table
    /// renders the two together.
    LastName,
    Gender,
    DateOfBirth,
    PlaceOfResidence,
}

impl BrpCheckedField {
    /// The label of the candidate-detail row this field belongs to, so a
    /// finding is named exactly like the data it is about.
    pub fn label(self, locale: Locale) -> String {
        match self {
            Self::Bsn => trans!("person.fields.bsn", locale),
            Self::Initials => trans!("person.fields.initials", locale),
            Self::LastName => trans!("person.fields.last_name", locale),
            Self::Gender => trans!("person.fields.gender", locale),
            Self::DateOfBirth => trans!("person.fields.date_of_birth", locale),
            Self::PlaceOfResidence => trans!("person.fields.place_of_residence", locale),
        }
    }
}

/// One thing the BRP check found for a single candidate.
///
/// Findings are recorded on the stream and shown next to the candidate's data.
/// They are deliberately **not** turned into omissions: per the central
/// committee use case the committee first confirms a BRP difference and may
/// correct it ambtshalve, and only what remains after that becomes a verzuim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrpFinding {
    /// The candidate's value differs from the value in the BRP.
    Mismatch {
        field: BrpCheckedField,
        brp_value: String,
    },
    /// The BRP returned a value this application cannot interpret, so the
    /// field could not be compared at all. Kept apart from [`Self::Mismatch`]
    /// so that a parsing problem is never reported as a data difference.
    Unparsable {
        field: BrpCheckedField,
        brp_value: String,
    },
    /// The BRP holds no value for a field that was requested.
    MissingInBrp { field: BrpCheckedField },
    /// No person in the BRP has this burgerservicenummer.
    BsnUnknown,
    /// More than one person in the BRP matched this burgerservicenummer.
    BsnNotUnique,
    /// The candidate has no burgerservicenummer, so no lookup was possible.
    BsnMissing,
    /// The candidate is recorded as deliberately having no burgerservicenummer.
    BsnNoneConfirmed,
    /// The BRP records a date of death for this candidate.
    Deceased { date_of_death: String },
    /// The candidate is not old enough to be elected.
    Underage { date_of_birth: String },
    /// The BRP holds no date of birth, so the candidate's age could not be
    /// established.
    AgeUnknown,
    /// The BRP records no Dutch nationality for this candidate.
    NotDutch,
    /// The BRP records this candidate as excluded from the right to vote.
    ExcludedFromSuffrage,
    /// The candidate lives abroad, so the BRP holds no `woonplaats`.
    ResidenceAbroad,
    /// The candidate's BRP residence is a location rather than an address
    /// (a briefadres, for instance), so the BRP holds no `woonplaats`.
    ResidenceWithoutAddress,
    /// The candidate's residence is unknown in the BRP.
    ResidenceUnknown,
}

impl BrpFinding {
    /// The candidate-detail field this finding belongs to, or `None` when it
    /// is about the candidate as a whole (their eligibility, or the lookup
    /// itself failing).
    pub fn field(&self) -> Option<BrpCheckedField> {
        match self {
            Self::Mismatch { field, .. }
            | Self::Unparsable { field, .. }
            | Self::MissingInBrp { field } => Some(*field),
            Self::ResidenceAbroad | Self::ResidenceWithoutAddress | Self::ResidenceUnknown => {
                Some(BrpCheckedField::PlaceOfResidence)
            }
            Self::BsnUnknown | Self::BsnNotUnique | Self::BsnMissing | Self::BsnNoneConfirmed => {
                Some(BrpCheckedField::Bsn)
            }
            Self::Deceased { .. } | Self::Underage { .. } | Self::AgeUnknown => {
                Some(BrpCheckedField::DateOfBirth)
            }
            Self::NotDutch | Self::ExcludedFromSuffrage => None,
        }
    }

    /// What the committee is shown for this finding.
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::Mismatch { field, brp_value } => {
                trans!(
                    "csb.brp.finding.mismatch",
                    locale,
                    field.label(locale),
                    brp_value
                )
            }
            Self::Unparsable { field, brp_value } => {
                trans!(
                    "csb.brp.finding.unparsable",
                    locale,
                    field.label(locale),
                    brp_value
                )
            }
            Self::MissingInBrp { field } => {
                trans!(
                    "csb.brp.finding.missing_in_brp",
                    locale,
                    field.label(locale)
                )
            }
            Self::BsnUnknown => trans!("csb.brp.finding.bsn_unknown", locale),
            Self::BsnNotUnique => trans!("csb.brp.finding.bsn_not_unique", locale),
            Self::BsnMissing => trans!("csb.brp.finding.bsn_missing", locale),
            Self::BsnNoneConfirmed => trans!("csb.brp.finding.bsn_none_confirmed", locale),
            Self::Deceased { date_of_death } => {
                trans!("csb.brp.finding.deceased", locale, date_of_death)
            }
            Self::Underage { date_of_birth } => {
                trans!("csb.brp.finding.underage", locale, date_of_birth)
            }
            Self::AgeUnknown => trans!("csb.brp.finding.age_unknown", locale),
            Self::NotDutch => trans!("csb.brp.finding.not_dutch", locale),
            Self::ExcludedFromSuffrage => {
                trans!("csb.brp.finding.excluded_from_suffrage", locale)
            }
            Self::ResidenceAbroad => trans!("csb.brp.finding.residence_abroad", locale),
            Self::ResidenceWithoutAddress => {
                trans!("csb.brp.finding.residence_without_address", locale)
            }
            Self::ResidenceUnknown => trans!("csb.brp.finding.residence_unknown", locale),
        }
    }
}
