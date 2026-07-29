use std::collections::HashSet;

use axum_extra::routing::TypedPath;

use crate::{
    AppError, CsbStore, Locale, QueryParamState,
    common::DateOfBirth,
    constants::DEFAULT_DATE_FORMAT,
    csb::{
        WithCorrections,
        examination::{
            extractors::CsbPoliticalGroup,
            structs::{CandidateCorrectionField, PaperCorrected},
        },
    },
    persons::{Person, PersonId},
    structs::csb::PersonCorrection,
    trans,
};

pub struct PaperCorrectedField {
    pub label: String,
    pub corrected: PaperCorrected,
    pub edit_path: String,
}

pub struct AllCsbCorrections {
    pub general: Vec<PaperCorrectedField>,
    pub candidates: Vec<CandidateCorrections>,
}

pub struct CandidateCorrections {
    /// the most "up-to-date" version of the Person, i.e. including all paper- and csb-corrections
    pub person: Person,
    pub corrections: Vec<PaperCorrectedField>,
}
impl CsbStore {
    pub fn get_all_corrections(
        &self,
        political_group: &CsbPoliticalGroup,
        locale: Locale,
    ) -> Result<AllCsbCorrections, AppError> {
        let mut candidates = Vec::new();
        for person in self.get_all_csb_corrected_persons() {
            candidates.push(self.compute_corrections(&person, political_group, locale)?)
        }

        let general = self
            .get_display_name_correction(political_group, locale)
            .into_iter()
            .collect();

        Ok(AllCsbCorrections {
            general,
            candidates,
        })
    }

    /// only return a set of [`PaperCorrected`] fields that are CSB corrected
    fn compute_corrections(
        &self,
        person: &PersonId,
        political_group: &CsbPoliticalGroup,
        locale: Locale,
    ) -> Result<CandidateCorrections, AppError> {
        // let mut csb_corrected = Vec::new();
        let imported = self
            .get_person(*person, WithCorrections::None)
            .ok_or(AppError::InternalServerError)?;
        let paper_corrected = self
            .get_person(*person, WithCorrections::Paper)
            .ok_or(AppError::InternalServerError)?;
        let fully_corrected = self
            .get_person(*person, WithCorrections::All)
            .ok_or(AppError::InternalServerError)?;

        type CandidatePaperCorrectedFieldInputs = (
            CandidateCorrectionField,
            Box<dyn Fn(&Person) -> String>,
            String,
        );

        let corrections = self
            .get_person_corrections(person)?
            .iter()
            .map(|correction| {
                let (correction_field, value_extractor, corrected_value): CandidatePaperCorrectedFieldInputs = match correction {
                    PersonCorrection::Initials(initials) => (
                        CandidateCorrectionField::Initials,
                        Box::new(|p: &Person| p.name.initials.to_string()),
                        initials.to_string(),
                    ),
                    PersonCorrection::LastName(last_name) => (
                        CandidateCorrectionField::LastName,
                        Box::new(|p: &Person| p.name.last_name.to_string()),
                        last_name.to_string(),
                    ),
                    PersonCorrection::DateOfBirth(date_of_birth) => (
                        CandidateCorrectionField::DateOfBirth,
                        Box::new(|p: &Person| {
                            DateOfBirth::format_option(&p.personal_data.date_of_birth)
                        }),
                        date_of_birth.format(DEFAULT_DATE_FORMAT).to_string(),
                    ),
                    PersonCorrection::PlaceOfResidence(place_of_residence) => (
                        CandidateCorrectionField::PlaceOfResidence,
                        Box::new(|p: &Person| {
                            p.personal_data
                                .place_of_residence
                                .as_ref()
                                .map(|place| place.to_string())
                                .unwrap_or_default()
                        }),
                        place_of_residence.to_string(),
                    ),
                };
                PaperCorrectedField {
                    label: correction_field.label(locale),
                    corrected: PaperCorrected::from_field(
                        Some(&imported),
                        Some(&paper_corrected),
                        value_extractor,
                    )
                    .with_csb_correction(Some(corrected_value)),
                    edit_path: political_group
                        .correction_person_path_from_all_rectifications(person, correction_field)
                        .to_string(),
                }
            })
            .collect();

        Ok(CandidateCorrections {
            person: fully_corrected,
            corrections,
        })
    }

    fn get_person_corrections(
        &self,
        person: &PersonId,
    ) -> Result<HashSet<PersonCorrection>, AppError> {
        Ok(self
            .data
            .read()
            .csb_corrected_persons
            .get(person)
            .ok_or(AppError::InternalServerError)?
            .get_corrections())
    }

    fn get_display_name_correction(
        &self,
        political_group: &CsbPoliticalGroup,
        locale: Locale,
    ) -> Option<PaperCorrectedField> {
        self.data
            .read()
            .csb_corrected_display_name
            .clone()
            .map(|name| PaperCorrectedField {
                label: trans!("political_group.display_name", locale),
                corrected: PaperCorrected::new(
                    self.get_display_name(WithCorrections::None),
                    self.get_display_name(WithCorrections::Paper),
                )
                .with_csb_correction(Some(name.to_string())),
                edit_path: political_group
                    .correction_display_name_path()
                    .with_query_params(QueryParamState::redirect_to(
                        political_group.all_restorations_path().to_string(),
                    ))
                    .to_string(),
            })
    }
}
