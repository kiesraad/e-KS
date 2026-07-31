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
    pub fn get_all_corrections(&self, locale: Locale) -> Result<AllCsbCorrections, AppError> {
        let political_group = CsbPoliticalGroup::new_from_csb_store(self);
        let mut candidates = Vec::new();
        for person in self.get_all_csb_corrected_persons() {
            candidates.push(self.compute_corrections(&person, &political_group, locale)?)
        }

        let general = self
            .get_display_name_correction(&political_group, locale)
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        CsbEvent::{self},
        common::{DisplayName, Initials, LastName, PlaceOfResidence},
        structs::csb::Correction,
        test_utils::sample_person,
    };

    use super::*;

    #[test]
    fn get_all_corrections_no_corrections() {
        let store = CsbStore::new_for_test();
        let locale = Locale::Nl;

        let corrections = store.get_all_corrections(locale).unwrap();

        assert_eq!(corrections.general.len(), 0);
        assert_eq!(corrections.candidates.len(), 0);
    }

    #[tokio::test]
    async fn get_all_corrections_two_candidates() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let locale = Locale::Nl;

        let p_id1 = PersonId::new();
        let p_id2 = PersonId::new();

        store.add_person(sample_person(p_id1));
        store.add_person(sample_person(p_id2));

        store
            .update(CsbEvent::UpdateCorrection(Correction::Person(
                p_id1,
                PersonCorrection::Initials(Initials::from_str("A.B.").unwrap()),
            )))
            .await?;
        store
            .update(CsbEvent::UpdateCorrection(Correction::Person(
                p_id1,
                PersonCorrection::LastName(LastName::from_str("Smit").unwrap()),
            )))
            .await?;
        store
            .update(CsbEvent::UpdateCorrection(Correction::Person(
                p_id2,
                PersonCorrection::PlaceOfResidence(PlaceOfResidence::Known(
                    "Amsterdam".to_string(),
                )),
            )))
            .await?;

        let corrections = store.get_all_corrections(locale)?;

        assert_eq!(corrections.general.len(), 0);
        assert_eq!(corrections.candidates.len(), 2);

        let p1_corrections = corrections
            .candidates
            .iter()
            .find(|c| c.person.id == p_id1)
            .unwrap();
        let p2_corrections = corrections
            .candidates
            .iter()
            .find(|c| c.person.id == p_id2)
            .unwrap();

        assert_eq!(p1_corrections.corrections.len(), 2);
        assert_eq!(p2_corrections.corrections.len(), 1);

        let p1_c1 = p1_corrections
            .corrections
            .iter()
            .find(|c| c.corrected.csb_corrected == Some("A.B.".to_string()))
            .unwrap();
        assert_eq!(
            p1_c1.edit_path,
            format!(
                "/csb/examination/{}/correction/person/{}/initials?&redirect_to=%2Fcsb%2Fexamination%2F{}%2Fomissions",
                store.stream_id, p_id1, store.stream_id
            )
        );
        assert_eq!(p1_c1.label, "Voorletters".to_string());

        let p1_c2 = p1_corrections
            .corrections
            .iter()
            .find(|c| c.corrected.csb_corrected == Some("Smit".to_string()))
            .unwrap();
        assert_eq!(
            p1_c2.edit_path,
            format!(
                "/csb/examination/{}/correction/person/{}/last-name?&redirect_to=%2Fcsb%2Fexamination%2F{}%2Fomissions",
                store.stream_id, p_id1, store.stream_id
            )
        );
        assert_eq!(p1_c2.label, "Achternaam".to_string());

        let p2_c1 = p2_corrections
            .corrections
            .iter()
            .find(|c| c.corrected.csb_corrected == Some("Amsterdam".to_string()))
            .unwrap();
        assert_eq!(
            p2_c1.edit_path,
            format!(
                "/csb/examination/{}/correction/person/{}/place-of-residence?&redirect_to=%2Fcsb%2Fexamination%2F{}%2Fomissions",
                store.stream_id, p_id2, store.stream_id
            )
        );
        assert_eq!(p2_c1.label, "Woonplaats".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn get_all_corrections_display_name() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let locale = Locale::Nl;

        store.data.write().csb_corrected_display_name =
            Some(DisplayName::from_str("Gecorrigeerde Partij").unwrap());

        let corrections = store.get_all_corrections(locale)?;

        assert_eq!(corrections.general.len(), 1);
        assert_eq!(corrections.candidates.len(), 0);

        let correction = &corrections.general[0];

        assert_eq!(
            correction.corrected.csb_corrected,
            Some("Gecorrigeerde Partij".to_string())
        );
        assert_eq!(
            correction.edit_path,
            format!(
                "/csb/examination/{}/correction/display-name?&redirect_to=%2Fcsb%2Fexamination%2F{}%2Fomissions",
                store.stream_id, store.stream_id
            )
        );
        assert_eq!(correction.label, "Geregistreerde aanduiding".to_string());

        Ok(())
    }
}
