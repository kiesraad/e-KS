use std::collections::HashSet;

use crate::{
    AppError, CsbStore,
    common::DateOfBirth,
    constants::DEFAULT_DATE_FORMAT,
    csb::{WithCorrections, examination::structs::PaperCorrected},
    persons::{Person, PersonId},
    structs::csb::PersonCorrection,
};

pub struct AllCsbCorrections {
    pub general: Vec<PaperCorrected>,
    pub candidates: Vec<CandidateCorrections>,
}

pub struct CandidateCorrections {
    /// the most "up-to-date" version of the Person, i.e. including all paper- and csb-corrections
    pub person: Person,
    pub corrections: Vec<PaperCorrected>,
}
impl CsbStore {
    pub fn get_all_corrections(&self) -> Result<AllCsbCorrections, AppError> {
        self.get_all_csb_corrected_persons()
            .iter()
            .map(|person| self.compute_corrections(person));

        Ok(AllCsbCorrections {
            general: vec![],
            candidates: vec![],
        })
    }

    /// only return a set of [`PaperCorrected`] fields that are CSB corrected
    fn compute_corrections(&self, person: &PersonId) -> Result<CandidateCorrections, AppError> {
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

        let corrections = self
            .get_person_corrections(person)?
            .iter()
            .map(|correction| match correction {
                PersonCorrection::Initials(initials) => {
                    PaperCorrected::from_field(Some(&imported), Some(&paper_corrected), |p| {
                        p.name.initials.to_string()
                    })
                    .with_csb_correction(Some(initials.to_string()))
                }
                PersonCorrection::LastName(last_name) => {
                    PaperCorrected::from_field(Some(&imported), Some(&paper_corrected), |p| {
                        p.name.last_name.to_string()
                    })
                    .with_csb_correction(Some(last_name.to_string()))
                }
                PersonCorrection::DateOfBirth(date_of_birth) => {
                    PaperCorrected::from_field(Some(&imported), Some(&paper_corrected), |p| {
                        DateOfBirth::format_option(&p.personal_data.date_of_birth)
                    })
                    .with_csb_correction(Some(
                        date_of_birth.format(DEFAULT_DATE_FORMAT).to_string(),
                    ))
                }
                PersonCorrection::PlaceOfResidence(place_of_residence) => {
                    PaperCorrected::from_field(Some(&imported), Some(&paper_corrected), |p| {
                        p.personal_data
                            .place_of_residence
                            .as_ref()
                            .map(|place| place.to_string())
                            .unwrap_or_default()
                    })
                    .with_csb_correction(Some(place_of_residence.to_string()))
                }
            })
            .collect();

        // FIXME
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
}
