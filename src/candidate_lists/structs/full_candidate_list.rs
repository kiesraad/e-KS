use serde::Serialize;

use crate::{
    candidate_lists::{Candidate, CandidateList, CandidateListId},
    persons::PersonId,
};

#[derive(Debug, Clone, Serialize)]
pub struct FullCandidateList {
    pub list: CandidateList,
    pub candidates: Vec<Candidate>,
}

impl FullCandidateList {
    pub fn get_index(&self, person_id: PersonId) -> Option<usize> {
        self.candidates
            .iter()
            .position(|c| c.person.id == person_id)
    }

    pub fn contains(&self, person_id: PersonId) -> bool {
        self.candidates.iter().any(|c| c.person.id == person_id)
    }

    pub fn get_ids(&self) -> Vec<PersonId> {
        self.candidates.iter().map(|c| c.person.id).collect()
    }

    pub fn id(&self) -> CandidateListId {
        self.list.id
    }

    pub fn update_position(&mut self, id: PersonId, position: usize) {
        dbg!("updating position", self.get_ids());

        let Some(current_index) = self.get_index(id) else {
            return;
        };

        let mut moved = self.candidates.remove(current_index);

        // convert the position (1, 2, 3...) to an index (0, 1, 2,..) and clamp it to the valid range
        let target_index = position.saturating_sub(1).min(self.candidates.len());

        moved.position = position;
        self.candidates.insert(target_index, moved);

        dbg!("moved candidate to position", current_index, target_index);
        dbg!("finished update position", self.get_ids());
    }
}
