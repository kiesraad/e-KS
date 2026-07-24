use crate::{CsbStore, csb::examination::structs::PaperCorrected, persons::Person};

pub struct AllCsbCorrections {
    pub general: Vec<PaperCorrected>,
    pub candidates: Vec<CandidateCorrections>
}

pub struct CandidateCorrections {
    /// the most "up-to-date" version of the Person, i.e. including all paper- and csb-corrections
    pub person: Person,
    pub corrections: Vec<PaperCorrected>
}
impl CsbStore {
    pub fn get_all_corrections(&self) -> AllCsbCorrections {
        self.get_all_csb_corrected_persons().iter().map(|person| self.compute_diff(person));

        
        AllCsbCorrections {general: vec![], candidates: vec![]}
    }

    fn compute_diff(&self, csb_corrected_person: &Person) -> CandidateCorrections {
        // let mut csb_corrected = Vec::new();
        let original = self.get_imported_or_corrected_person(csb_corrected_person.id);
        
        // FIXME
        CandidateCorrections { person: original.unwrap(), corrections: vec![] }
    }
}
