mod event;
mod event_info;
mod extractor;
mod getters;

pub use event::AppEvent;
pub(crate) use extractor::request_extractor;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    AppError, CsbEvent, CsbStore, Scope,
    candidate_lists::{CandidateList, CandidateListId},
    common::UtcDateTime,
    list_submitters::ListSubmitter,
    name_authorisations::{NameAuthorisation, NameAuthorisationId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    store::{Store, StoreData, StoreEvent},
};

/// Event-sourced domain projection for a single (stream, election) pair.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppStoreData {
    pub(crate) political_group: PoliticalGroup,
    pub(crate) persons: HashMap<PersonId, Person>,
    pub(crate) candidate_lists: HashMap<CandidateListId, CandidateList>,
    pub(crate) name_authorisations: HashMap<NameAuthorisationId, NameAuthorisation>,
    pub(crate) list_submitter: ListSubmitter,
    pub(crate) substitute_submitters: Vec<ListSubmitter>,
    pub(crate) events: Vec<StoreEvent<AppEvent>>,
}

impl StoreData for AppStoreData {
    type Event = AppEvent;

    fn apply(&mut self, event: StoreEvent<AppEvent>) {
        self.events.push(event.clone());

        let event_time = UtcDateTime::from(event.created_at);

        match event.payload {
            AppEvent::UpdatePoliticalGroup(pg) => self.political_group = pg,

            event @ (AppEvent::CreatePerson(_)
            | AppEvent::CreatePersonPersonalData { .. }
            | AppEvent::UpdatePerson(_)
            | AppEvent::UpdatePersonPersonalData { .. }
            | AppEvent::UpdatePersonAddress { .. }
            | AppEvent::UpdatePersonRepresentative { .. }
            | AppEvent::DeletePerson { .. }) => self.apply_person_event(event, event_time),

            event @ (AppEvent::CreateCandidateList(_)
            | AppEvent::UpdateCandidateListDistricts { .. }
            | AppEvent::UpdateCandidateListOrder { .. }
            | AppEvent::AddCandidateToCandidateList { .. }
            | AppEvent::RemoveCandidateFromCandidateList { .. }
            | AppEvent::DeleteCandidateList(_)) => {
                self.apply_candidate_list_event(event, event_time)
            }

            event @ (AppEvent::CreateNameAuthorisation(_)
            | AppEvent::UpdateNameAuthorisation(_)
            | AppEvent::DeleteNameAuthorisation(_)) => self.apply_name_authorisation_event(event),

            event @ (AppEvent::UpdateListSubmitter(_)
            | AppEvent::CreateSubstituteSubmitter(_)
            | AppEvent::UpdateSubstituteSubmitter(_)
            | AppEvent::DeleteSubstituteSubmitter { .. }) => self.apply_submitter_event(event),

            AppEvent::ImportCandidates {
                list_id,
                created_persons,
                updated_persons,
                candidates,
                ..
            } => self.apply_import_candidates_event(
                list_id,
                created_persons,
                updated_persons,
                candidates,
                event_time,
            ),

            // Only the serialized event is relevant for logging.
            AppEvent::DeveloperLogin { .. }
            | AppEvent::DownloadFile { .. }
            | AppEvent::HideDownloadWarning
            | AppEvent::ExportCsv { .. } => {}
        }
    }

    fn events(&self) -> &[StoreEvent<Self::Event>] {
        &self.events
    }

    fn scope() -> Scope {
        Scope::PoliticalGroup
    }
}

impl AppStoreData {
    /// Build a point-in-time snapshot of the projection by replaying `events`
    /// up to and including the event with `target_event_id`.
    ///
    /// The returned snapshot has its own `events` log
    /// cleared: it captures only the derived state, not the event history that
    /// produced it. `events` is expected to be the ordered event log of a single
    /// stream (as returned by `StoreData::events`); later events are ignored.
    pub fn snapshot_until(events: &[StoreEvent<AppEvent>], target_event_id: usize) -> Self {
        let mut snapshot = AppStoreData::default();
        for event in events
            .iter()
            .filter(|event| event.event_id <= target_event_id)
        {
            snapshot.apply(event.clone());
        }
        snapshot.events.clear();
        snapshot
    }

    /// Apply a person-related event. Routed here exclusively by [`Self::apply`].
    fn apply_person_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
            AppEvent::CreatePerson(_) | AppEvent::CreatePersonPersonalData { .. } => {
                self.apply_person_create_event(event, event_time)
            }
            AppEvent::UpdatePerson(_)
            | AppEvent::UpdatePersonPersonalData { .. }
            | AppEvent::UpdatePersonAddress { .. }
            | AppEvent::UpdatePersonRepresentative { .. } => {
                self.apply_person_update_event(event, event_time)
            }
            AppEvent::DeletePerson { person_id } => {
                self.candidate_lists
                    .values_mut()
                    .for_each(|list| list.candidates.retain(|id| *id != person_id));

                self.persons.remove(&person_id);
            }
            _ => unreachable!("apply_person_event received a non-person event"),
        }
    }

    /// Apply a person-creation event. Routed here exclusively by [`Self::apply_person_event`].
    fn apply_person_create_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
            AppEvent::CreatePerson(mut person) => {
                person.updated_at = event_time;
                self.persons.insert(person.id, person);
            }
            AppEvent::CreatePersonPersonalData {
                person_id,
                name,
                personal_data,
            } => {
                let person = Person {
                    id: person_id,
                    name,
                    personal_data,
                    updated_at: event_time,
                    ..Default::default()
                };
                self.persons.insert(person_id, person);
            }
            _ => unreachable!("apply_person_create_event received a non-create event"),
        }
    }

    /// Apply a person-update event. Routed here exclusively by [`Self::apply_person_event`].
    fn apply_person_update_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
            AppEvent::UpdatePerson(mut person) => {
                person.updated_at = event_time;
                let person_id = person.id;
                self.persons.entry(person_id).and_modify(|existing| {
                    *existing = person;
                });
            }
            AppEvent::UpdatePersonPersonalData {
                person_id,
                name,
                personal_data,
            } => {
                self.persons.entry(person_id).and_modify(|existing| {
                    existing.name = name;
                    existing.personal_data = personal_data;
                    existing.updated_at = event_time;
                });
            }
            AppEvent::UpdatePersonAddress { person_id, address } => {
                self.persons.entry(person_id).and_modify(|existing| {
                    existing.address = address;
                    existing.updated_at = event_time;
                });
            }
            AppEvent::UpdatePersonRepresentative {
                person_id,
                representative,
            } => {
                self.persons.entry(person_id).and_modify(|existing| {
                    existing.representative = representative;
                    existing.updated_at = event_time;
                });
            }
            _ => unreachable!("apply_person_update_event received a non-update event"),
        }
    }

    /// Apply a candidate-list event. Routed here exclusively by [`Self::apply`].
    fn apply_candidate_list_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
            AppEvent::CreateCandidateList(mut cl) => {
                cl.created_at = event_time;
                self.candidate_lists.insert(cl.id, cl);
            }
            AppEvent::UpdateCandidateListDistricts {
                list_id,
                electoral_districts,
            } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.electoral_districts = electoral_districts;
                });
            }
            AppEvent::UpdateCandidateListOrder {
                list_id,
                candidates,
            } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.candidates = candidates;
                });
            }
            AppEvent::AddCandidateToCandidateList { list_id, person_id } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    if !existing.candidates.contains(&person_id) {
                        existing.candidates.push(person_id);
                    }
                });
            }
            AppEvent::RemoveCandidateFromCandidateList { list_id, person_id } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.candidates.retain(|id| *id != person_id);
                });
            }
            AppEvent::DeleteCandidateList(cl_id) => {
                self.candidate_lists.remove(&cl_id);
            }
            _ => unreachable!("apply_candidate_list_event received a non-candidate-list event"),
        }
    }

    /// Apply a name-authorisation event. Routed here exclusively by [`Self::apply`].
    fn apply_name_authorisation_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::CreateNameAuthorisation(aa) => {
                self.name_authorisations.insert(aa.id, aa);
            }
            AppEvent::UpdateNameAuthorisation(aa) => {
                let aa_id = aa.id;
                self.name_authorisations
                    .entry(aa_id)
                    .and_modify(|existing| {
                        *existing = aa;
                    });
            }
            AppEvent::DeleteNameAuthorisation(aa_id) => {
                self.name_authorisations.remove(&aa_id);
            }
            _ => unreachable!(
                "apply_name_authorisation_event received a non-name-authorisation event"
            ),
        }
    }

    /// Apply a list-submitter event. Routed here exclusively by [`Self::apply`].
    fn apply_submitter_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::UpdateListSubmitter(ls) => {
                self.list_submitter = ls;
            }
            AppEvent::CreateSubstituteSubmitter(ss) => {
                self.substitute_submitters.push(ss);
            }
            AppEvent::UpdateSubstituteSubmitter(ss) => {
                let ss_id = ss.id;
                if let Some(existing) = self
                    .substitute_submitters
                    .iter_mut()
                    .find(|existing| existing.id == ss_id)
                {
                    *existing = ss;
                }
            }
            AppEvent::DeleteSubstituteSubmitter {
                substitute_submitter_id: ss_id,
            } => {
                self.substitute_submitters
                    .retain(|submitter| submitter.id != ss_id);
            }
            _ => unreachable!("apply_submitter_event received a non-submitter event"),
        }
    }

    /// Apply an import-candidates event. Routed here exclusively by [`Self::apply`].
    fn apply_import_candidates_event(
        &mut self,
        list_id: CandidateListId,
        created_persons: Vec<Person>,
        updated_persons: Vec<Person>,
        candidates: Vec<PersonId>,
        event_time: UtcDateTime,
    ) {
        for mut person in created_persons {
            person.updated_at = event_time;
            self.persons.insert(person.id, person);
        }
        for mut person in updated_persons {
            person.updated_at = event_time;
            let person_id = person.id;
            self.persons.entry(person_id).and_modify(|existing| {
                *existing = person;
            });
        }
        self.candidate_lists.entry(list_id).and_modify(|existing| {
            existing.candidates = candidates;
        });
    }
}

/// Store handle used by the app feature handlers: reads come from the
/// [`AppStoreData`] projection, writes dispatch [`AppEvent`]s to the write
/// target.
///
/// A political group session appends events to its own stream. A CSB session
/// correcting the paper documents gets every event wrapped in
/// [`CsbEvent::PaperCorrectedUpdate`] and appended to the CSB stream instead,
/// so corrections land in that stream's `paper_corrected_data` projection and
/// the political group's own stream is never touched.
#[derive(Clone)]
pub struct AppStore {
    /// Projection served to reads; [`std::ops::Deref`] keeps `store.election`,
    /// `store.data` and the getters working unchanged.
    projection: Store<AppStoreData>,
    target: WriteTarget,
}

#[derive(Clone)]
enum WriteTarget {
    /// Persist events on the projection's own stream.
    Own,
    /// Wrap events in [`CsbEvent::PaperCorrectedUpdate`] and persist them on
    /// this CSB stream. The projection is a request-local snapshot of the
    /// stream's `paper_corrected_data`.
    PaperCorrections(CsbStore),
}

impl std::ops::Deref for AppStore {
    type Target = Store<AppStoreData>;

    fn deref(&self) -> &Self::Target {
        &self.projection
    }
}

impl AppStore {
    /// Wrap a store that persists app events on its own stream.
    pub fn own(store: Store<AppStoreData>) -> Self {
        Self {
            projection: store,
            target: WriteTarget::Own,
        }
    }

    /// Build a paper-corrections handle over a loaded CSB store: reads serve a
    /// snapshot of its `paper_corrected_data`, writes go to the CSB stream.
    pub fn paper_corrections(csb_store: CsbStore) -> Self {
        let projection = Store::new_for_temp_stream(csb_store.election);
        *projection.data.write() = csb_store.data.read().paper_corrected_data.clone();

        Self {
            projection: Store {
                stream_id: csb_store.stream_id,
                ..projection
            },
            target: WriteTarget::PaperCorrections(csb_store),
        }
    }

    /// Create a temporary, in-memory handle with no persistence.
    pub fn new_for_temp_stream(election: crate::ElectionConfig) -> Self {
        Self::own(Store::new_for_temp_stream(election))
    }

    /// Persist an event on the write target and apply it to the projection.
    pub async fn update(&self, event: AppEvent) -> Result<(), AppError> {
        match &self.target {
            WriteTarget::Own => self.projection.update(event).await,
            WriteTarget::PaperCorrections(csb_store) => {
                csb_store
                    .update(CsbEvent::PaperCorrectedUpdate(Box::new(event)))
                    .await?;

                // Refresh the snapshot so reads later in this request observe
                // the correction.
                *self.projection.data.write() = csb_store.data.read().paper_corrected_data.clone();

                Ok(())
            }
        }
    }

    /// The CSB stream receiving paper corrections, when this handle is in
    /// paper-corrections mode.
    pub fn paper_corrections_stream_id(&self) -> Option<crate::StreamId> {
        match &self.target {
            WriteTarget::Own => None,
            WriteTarget::PaperCorrections(csb_store) => Some(csb_store.stream_id),
        }
    }
}

#[cfg(all(test, feature = "database"))]
impl AppStore {
    pub async fn new_with_pool_for_stream(
        pool: sqlx::PgPool,
        stream_id: crate::StreamId,
        election: crate::ElectionConfig,
        encryption: &crate::store::EventEncryption,
    ) -> Result<Self, AppError> {
        Ok(Self::own(
            Store::new_with_pool_for_stream(pool, stream_id, election, encryption).await?,
        ))
    }
}

#[cfg(test)]
impl AppStore {
    pub fn new_for_test() -> Self {
        Self::new_for_test_with_election(crate::ElectionConfig::EK27)
    }

    pub fn new_for_test_with_election(election: crate::ElectionConfig) -> Self {
        use crate::StreamId;

        let data = AppStoreData {
            political_group: crate::test_utils::sample_political_group(),
            ..AppStoreData::default()
        };

        Self::own(crate::store::Store {
            stream_id: StreamId::new(),
            election,
            backend: crate::store::persistence::StoreBackend::Memory {
                store: crate::store::memory::MemoryStore::default(),
            },
            data: std::sync::Arc::new(parking_lot::RwLock::new(data)),
        })
    }
}
