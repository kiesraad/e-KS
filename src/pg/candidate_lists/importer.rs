use eml_nl::{
    documents::EML,
    io::{EMLParsingMode, EMLRead},
};

use crate::{
    AppError, Locale, PgEvent, PgStore,
    candidate_lists::{CSV_HEADERS, CandidateRecord, CandidateRecordCsv},
    core::{Csv, CsvError, translated_field_name},
    form::FieldErrors,
    structs::{
        candidate_lists::CandidateList,
        common::{Bsn, BsnOrNoneConfirmed},
        persons::{Person, PersonId},
    },
    trans,
};

#[derive(Debug)]
pub(crate) enum ImportCandidateListError {
    App(AppError),
    Messages(Vec<String>),
}

impl From<AppError> for ImportCandidateListError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}

#[derive(Clone)]
struct PreparedPerson {
    person: Person,
    exists: bool,
}

/// Where a failing record sits in the uploaded file. A CSV row is reported by
/// its line (the header takes line 1), an EML candidate by its position in the
/// nomination.
#[derive(Clone, Copy)]
enum RecordLocation {
    CsvLine,
    EmlPosition,
}

impl RecordLocation {
    /// Translate one field error for the record with the given zero-based index.
    fn message(self, index: usize, field_name: String, message: String, locale: Locale) -> String {
        match self {
            RecordLocation::CsvLine => CsvError::ParseError {
                line_number: index + 2,
                field_name,
                message,
            }
            .message(locale),
            RecordLocation::EmlPosition => trans!(
                "candidate_list.import_errors.eml.parse_error",
                locale,
                index + 1,
                translated_field_name(&field_name, locale),
                message
            ),
        }
    }
}

pub(crate) async fn import_candidate_list_csv(
    list: &mut CandidateList,
    store: &PgStore,
    csv_data: &[u8],
    locale: Locale,
    file_name: String,
    file_size: usize,
) -> Result<ImportOutcome, ImportCandidateListError> {
    ensure_expected_headers(csv_data, locale)?;
    let records = parse_records(csv_data, locale)?;

    import_records(
        list,
        store,
        records,
        RecordLocation::CsvLine,
        locale,
        file_name,
        file_size,
    )
    .await
}

/// Import the candidates of an EML 2.10 nomination document, the same file this
/// application exports. Only the candidates are read: the affiliation, election
/// and proposers in the document are ignored, exactly like the CSV import.
pub(crate) async fn import_candidate_list_eml(
    list: &mut CandidateList,
    store: &PgStore,
    eml_data: &[u8],
    locale: Locale,
    file_name: String,
    file_size: usize,
) -> Result<ImportOutcome, ImportCandidateListError> {
    let records = parse_eml_records(eml_data, locale)?;

    import_records(
        list,
        store,
        records,
        RecordLocation::EmlPosition,
        locale,
        file_name,
        file_size,
    )
    .await
}

/// Validate the parsed records and store them as the list's candidates.
async fn import_records(
    list: &mut CandidateList,
    store: &PgStore,
    mut records: Vec<CandidateRecord>,
    location: RecordLocation,
    locale: Locale,
    file_name: String,
    file_size: usize,
) -> Result<ImportOutcome, ImportCandidateListError> {
    let capped = records.len() > store.candidate_limit();
    records.truncate(store.candidate_limit());

    let persons = collect_persons(records, store.get_persons(), location, locale)?;
    emit_import_event(list, store, persons, file_name, file_size).await?;

    Ok(ImportOutcome { capped })
}

/// Information about a successful import that the caller surfaces to the user.
#[derive(Debug)]
pub(crate) struct ImportOutcome {
    /// The number of candidates in the file exceeded the store's candidate
    /// limit and the list was truncated to that maximum.
    pub capped: bool,
}

fn ensure_expected_headers(data: &[u8], locale: Locale) -> Result<(), ImportCandidateListError> {
    if has_expected_headers(data) {
        Ok(())
    } else {
        Err(ImportCandidateListError::Messages(vec![trans!(
            "candidate_list.import_errors.invalid_headers",
            locale
        )]))
    }
}

fn has_expected_headers(data: &[u8]) -> bool {
    let mut reader = crate::core::reader_from_bytes(data);
    match reader.headers() {
        Ok(headers) => headers.iter().eq(CSV_HEADERS),
        Err(_) => false,
    }
}

fn parse_records(
    data: &[u8],
    locale: Locale,
) -> Result<Vec<CandidateRecord>, ImportCandidateListError> {
    Csv::<CandidateRecordCsv>::from_bytes(data)
        .map(|records| records.into_iter().map(CandidateRecord::from).collect())
        .map_err(|errors| ImportCandidateListError::Messages(csv_error_messages(errors, locale)))
}

/// Read the candidates from an EML 2.10 nomination document.
///
/// Values the EML reader cannot parse (a malformed date, say) are kept as raw
/// strings by [`EMLParsingMode::StrictFallback`], so they end up as ordinary
/// field errors on the candidate they belong to instead of failing the whole
/// upload with an XML-level message.
fn parse_eml_records(
    data: &[u8],
    locale: Locale,
) -> Result<Vec<CandidateRecord>, ImportCandidateListError> {
    let xml = std::str::from_utf8(data)
        .map_err(|_| {
            eml_error(trans!(
                "candidate_list.import_errors.eml.invalid_encoding",
                locale
            ))
        })?
        .trim_start_matches('\u{feff}');

    let document = EML::parse_eml(xml, EMLParsingMode::StrictFallback)
        .ok()
        .map_err(|error| {
            eml_error(trans!(
                "candidate_list.import_errors.eml.invalid_xml",
                locale,
                error
            ))
        })?;

    let nomination = document.as_nomination_doc().ok_or_else(|| {
        eml_error(trans!(
            "candidate_list.import_errors.eml.not_a_nomination",
            locale,
            document.to_eml_id()
        ))
    })?;

    Ok(nomination
        .nomination_data
        .affiliation
        .candidates
        .iter()
        .map(CandidateRecord::from)
        .collect())
}

fn eml_error(message: String) -> ImportCandidateListError {
    ImportCandidateListError::Messages(vec![message])
}

fn csv_error_messages(errors: Vec<CsvError>, locale: Locale) -> Vec<String> {
    errors
        .into_iter()
        .map(|error| error.message(locale))
        .collect()
}

fn collect_persons(
    records: Vec<CandidateRecord>,
    existing_persons: Vec<Person>,
    location: RecordLocation,
    locale: Locale,
) -> Result<Vec<PreparedPerson>, ImportCandidateListError> {
    let mut prepared_people = Vec::new();
    let mut errors = Vec::new();

    for (index, record) in records.into_iter().enumerate() {
        match validate_record(record, index, location, locale) {
            Ok(person) => upsert_person(person, &mut prepared_people, &existing_persons),
            Err(ImportCandidateListError::Messages(messages)) => errors.extend(messages),
            Err(error) => return Err(error),
        }
    }

    if errors.is_empty() {
        Ok(prepared_people)
    } else {
        Err(ImportCandidateListError::Messages(errors))
    }
}

fn validate_record(
    record: CandidateRecord,
    index: usize,
    location: RecordLocation,
    locale: Locale,
) -> Result<Person, ImportCandidateListError> {
    record
        .validate_create()
        .map(refresh_bag_checks)
        .map_err(|error| {
            ImportCandidateListError::Messages(field_error_messages(
                index,
                error.errors(),
                location,
                locale,
            ))
        })
}

/// Imported addresses bypass the address forms, so refresh their BAG flags here.
fn refresh_bag_checks(mut person: Person) -> Person {
    person.address.update_is_known_in_bag();
    if let Some(representative) = &mut person.representative {
        representative.address.update_is_known_in_bag();
    }
    person
}

fn upsert_person(
    person: Person,
    prepared_people: &mut Vec<PreparedPerson>,
    existing_persons: &[Person],
) {
    match find_prepared_person(&person, prepared_people) {
        Some(prepared) => update_prepared_person(prepared, person),
        None => prepared_people.push(prepare_person(person, existing_persons)),
    }
}

fn find_prepared_person<'a>(
    person: &Person,
    prepared_people: &'a mut [PreparedPerson],
) -> Option<&'a mut PreparedPerson> {
    prepared_people
        .iter_mut()
        .find(|prepared| same_import_identity(person, &prepared.person))
}

fn person_bsn(person: &Person) -> Option<&Bsn> {
    match &person.personal_data.bsn {
        Some(BsnOrNoneConfirmed::Bsn(bsn)) => Some(bsn),
        _ => None,
    }
}

fn update_prepared_person(prepared: &mut PreparedPerson, person: Person) {
    prepared.person = Person {
        id: prepared.person.id,
        ..person
    };
}

fn same_import_identity(person: &Person, existing_person: &Person) -> bool {
    match (person_bsn(person), person_bsn(existing_person)) {
        (Some(person_bsn), Some(existing_bsn)) => person_bsn == existing_bsn,
        (None, None) => same_initials_and_last_name(person, existing_person),
        _ => false,
    }
}

fn same_initials_and_last_name(person: &Person, existing_person: &Person) -> bool {
    person.name.initials == existing_person.name.initials
        && person.name.last_name == existing_person.name.last_name
}

fn find_matching_person<'a>(person: &Person, existing_persons: &'a [Person]) -> Option<&'a Person> {
    existing_persons
        .iter()
        .find(|existing_person| same_import_identity(person, existing_person))
}

fn prepare_person(person: Person, existing_persons: &[Person]) -> PreparedPerson {
    match find_matching_person(&person, existing_persons) {
        Some(existing_person) => PreparedPerson {
            person: Person {
                id: existing_person.id,
                ..person
            },
            exists: true,
        },
        None => PreparedPerson {
            person: Person {
                id: PersonId::new(),
                ..person
            },
            exists: false,
        },
    }
}

fn field_error_messages(
    index: usize,
    errors: FieldErrors,
    location: RecordLocation,
    locale: Locale,
) -> Vec<String> {
    errors
        .into_iter()
        .map(|(field_name, error)| {
            location.message(index, field_name, error.message(locale), locale)
        })
        .collect()
}

async fn emit_import_event(
    list: &mut CandidateList,
    store: &PgStore,
    persons: Vec<PreparedPerson>,
    file_name: String,
    file_size: usize,
) -> Result<(), ImportCandidateListError> {
    let candidates = persons.iter().map(|p| p.person.id).collect::<Vec<_>>();

    let mut created_persons = Vec::new();
    let mut updated_persons = Vec::new();
    for prepared in persons {
        if prepared.exists {
            updated_persons.push(prepared.person);
        } else {
            created_persons.push(prepared.person);
        }
    }

    store
        .update(PgEvent::ImportCandidates {
            list_id: list.id,
            file_name,
            file_size,
            created_persons,
            updated_persons,
            candidates,
        })
        .await?;

    *list = store.get_candidate_list(list.id)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        MAX_CANDIDATES,
        structs::{candidate_lists::CandidateListId, persons::PersonId},
        test_utils::{sample_candidate_list, sample_person, sample_person_with},
    };

    #[tokio::test]
    async fn reuses_existing_person() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let existing_person = sample_person(PersonId::new());

        existing_person.create(&store).await?;
        list.create(&store).await?;

        import_candidate_list_csv(
            &mut list,
            &store,
            valid_csv().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        assert_eq!(store.get_person_count(), 1);
        assert_eq!(
            store.get_candidate_list(list_id)?.candidates,
            vec![existing_person.id]
        );
        assert_eq!(
            store
                .get_person(existing_person.id)?
                .name
                .first_name
                .as_deref()
                .map(ToString::to_string),
            Some("Henk".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn reuses_no_bsn_match_by_initials_and_last_name() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let mut existing_person =
            sample_person_with(PersonId::new(), Some("Hendrik"), "Jansen", None, "H.A.H.A.");
        existing_person.personal_data.bsn = None;

        existing_person.create(&store).await?;
        list.create(&store).await?;

        import_candidate_list_csv(
            &mut list,
            &store,
            no_bsn_csv_with_different_first_name().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        assert_eq!(store.get_person_count(), 1);
        assert_eq!(
            store.get_candidate_list(list_id)?.candidates,
            vec![existing_person.id]
        );

        Ok(())
    }

    #[tokio::test]
    async fn does_not_reuse_bsn_match_for_no_bsn_import() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let mut existing_person =
            sample_person_with(PersonId::new(), Some("Hendrik"), "Jansen", None, "H.A.H.A.");
        existing_person.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("123456782".parse().expect("bsn")));

        existing_person.create(&store).await?;
        list.create(&store).await?;

        import_candidate_list_csv(
            &mut list,
            &store,
            no_bsn_csv_with_different_first_name().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        let candidates = store.get_candidate_list(list_id)?.candidates;
        let persisted_existing_person = store.get_person(existing_person.id)?;

        assert_eq!(store.get_person_count(), 2);
        assert_eq!(candidates.len(), 1);
        assert_ne!(candidates[0], existing_person.id);
        assert_eq!(persisted_existing_person.id, existing_person.id);
        assert_eq!(
            persisted_existing_person.personal_data.bsn,
            existing_person.personal_data.bsn
        );

        Ok(())
    }

    #[tokio::test]
    async fn allows_same_name_when_only_one_row_has_bsn() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        list.create(&store).await?;

        import_candidate_list_csv(
            &mut list,
            &store,
            mixed_bsn_duplicate_name_csv().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        assert_eq!(store.get_person_count(), 2);
        assert_eq!(store.get_candidate_list(list_id)?.candidates.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn merges_no_bsn_duplicates_by_initials_and_last_name() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        list.create(&store).await?;

        import_candidate_list_csv(
            &mut list,
            &store,
            duplicate_no_bsn_csv().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("duplicate rows should merge");

        let candidate_id = store.get_candidate_list(list_id)?.candidates[0];

        assert_eq!(store.get_person_count(), 1);
        assert_eq!(store.get_candidate_list(list_id)?.candidates.len(), 1);
        assert_eq!(
            store
                .get_person(candidate_id)?
                .name
                .first_name
                .as_deref()
                .map(ToString::to_string),
            Some("Hendrik".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn merges_duplicates_by_bsn() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        list.create(&store).await?;

        import_candidate_list_csv(
            &mut list,
            &store,
            duplicate_bsn_csv().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("duplicate rows should merge");

        let candidate_id = store.get_candidate_list(list_id)?.candidates[0];

        assert_eq!(store.get_person_count(), 1);
        assert_eq!(store.get_candidate_list(list_id)?.candidates.len(), 1);
        assert_eq!(
            store
                .get_person(candidate_id)?
                .name
                .first_name
                .as_deref()
                .map(ToString::to_string),
            Some("Hendrik".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn emits_a_single_event_for_the_whole_import() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        list.create(&store).await?;
        let event_id_before_import = store.current_event_id();

        import_candidate_list_csv(
            &mut list,
            &store,
            mixed_bsn_duplicate_name_csv().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        assert_eq!(store.current_event_id(), event_id_before_import + 1);
        assert_eq!(store.get_person_count(), 2);
        assert_eq!(store.get_candidate_list(list_id)?.candidates.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn import_caps_candidates_at_max() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let mut csv = format!("{}\r\n", csv_headers());
        for index in 0..(MAX_CANDIDATES + 5) {
            csv.push_str(&format!(
                "H.A.H.A.,Henk,,Jansen{index},Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
            ));
        }

        let outcome = import_candidate_list_csv(
            &mut list,
            &store,
            csv.as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        assert!(outcome.capped);
        assert_eq!(
            store.get_candidate_list(list_id)?.candidates.len(),
            MAX_CANDIDATES
        );
        // Rows past the cap are dropped entirely: no person records may be
        // persisted for them.
        assert_eq!(store.get_person_count(), MAX_CANDIDATES);

        Ok(())
    }

    #[tokio::test]
    async fn import_runs_bag_check_on_correspondence_address() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        list.create(&store).await?;

        let csv = format!(
            "{}\r\n{}{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Amsterdam,NL,kandidaat heeft geen BSN,01-02-1990,v,1012JS,1,,Dam,Amsterdam,,,,,,,,\r\n",
            "H.A.H.A.,Piet,,Pietersen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        );

        import_candidate_list_csv(
            &mut list,
            &store,
            csv.as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        let candidates = store.get_candidate_list(list_id)?.candidates;
        let known = store.get_person(candidates[0])?;
        let unknown = store.get_person(candidates[1])?;

        assert_eq!(known.address.known_in_bag, Some(true));
        assert_eq!(unknown.address.known_in_bag, Some(false));

        Ok(())
    }

    #[tokio::test]
    async fn import_runs_bag_check_on_representative_address() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        list.create(&store).await?;

        let csv = format!(
            "{}\r\n{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Antwerp,BE,kandidaat heeft geen BSN,01-02-1990,v,,,,,,P.,,Puk,1012JS,1,,Dam,Amsterdam\r\n"
        );

        import_candidate_list_csv(
            &mut list,
            &store,
            csv.as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await
        .expect("import should succeed");

        let candidate_id = store.get_candidate_list(list_id)?.candidates[0];
        let representative = store
            .get_person(candidate_id)?
            .representative
            .expect("representative should be present");

        assert_eq!(representative.address.known_in_bag, Some(true));

        Ok(())
    }

    #[tokio::test]
    async fn returns_all_row_validation_errors() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let mut list = sample_candidate_list(CandidateListId::new());

        list.create(&store).await?;

        let result = import_candidate_list_csv(
            &mut list,
            &store,
            multiple_invalid_rows_csv().as_bytes(),
            Locale::En,
            "test.csv".to_string(),
            0,
        )
        .await;

        match result {
            Err(ImportCandidateListError::Messages(messages)) => {
                assert_eq!(messages.len(), 2);
                assert!(messages.iter().any(|message| message.contains("line 2")));
                assert!(messages.iter().any(|message| message.contains("line 3")));
            }
            other => panic!("expected validation messages, got {other:?}"),
        }

        assert_eq!(store.get_person_count(), 0);

        Ok(())
    }

    /// The EML 2.10 export reads back as the same candidates: the persons are
    /// matched to the existing ones and the list is filled in the same order.
    #[tokio::test]
    async fn imports_the_eml_export_of_a_list() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let person = sample_person(PersonId::new());
        person.create(&store).await?;

        let eml = export_eml(&store, person.id).await?;

        let mut list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        import_candidate_list_eml(
            &mut list,
            &store,
            &eml,
            Locale::En,
            "eml210.eml.xml".to_string(),
            eml.len(),
        )
        .await
        .expect("import should succeed");

        // The candidate matches the person that was exported, so no second
        // person is created.
        assert_eq!(store.get_person_count(), 1);
        assert_eq!(
            store.get_candidate_list(list.id)?.candidates,
            vec![person.id]
        );

        assert_same_candidate(&store.get_person(person.id)?, &person);

        // EML 210 has no equivalent of the "candidate has no BSN"
        // confirmation, so that confirmation does not survive the round trip.
        assert_eq!(
            person.personal_data.bsn,
            Some(BsnOrNoneConfirmed::NoneConfirmed)
        );
        assert_eq!(store.get_person(person.id)?.personal_data.bsn, None);

        Ok(())
    }

    /// Export a candidate list holding one person as EML 2.10.
    async fn export_eml(store: &PgStore, person_id: PersonId) -> Result<Vec<u8>, AppError> {
        use crate::{
            ElectionConfig, core::ModelLocale, structs::list_submitters::ListSubmitterId,
            test_utils::sample_list_submitter,
        };

        sample_list_submitter(ListSubmitterId::new())
            .update(store)
            .await?;

        let mut list = sample_candidate_list(CandidateListId::new());
        list.candidates.push(person_id);
        list.create(store).await?;

        crate::models::eml210::eml210(
            store,
            &ElectionConfig::EK27,
            &store.get_political_group(),
            list.id,
            ModelLocale::Nl,
        )
    }

    /// Everything about a candidate that EML 2.10 carries, compared by value.
    #[track_caller]
    fn assert_same_candidate(imported: &Person, expected: &Person) {
        use crate::test_utils::display_opt;

        assert_eq!(imported.name, expected.name);
        assert_eq!(
            imported.personal_data.date_of_birth,
            expected.personal_data.date_of_birth
        );
        assert_eq!(imported.personal_data.gender, expected.personal_data.gender);
        assert_eq!(
            display_opt(&imported.personal_data.place_of_residence),
            display_opt(&expected.personal_data.place_of_residence)
        );
        assert_eq!(
            display_opt(&imported.personal_data.country),
            display_opt(&expected.personal_data.country)
        );
        assert_eq!(
            display_opt(&imported.address.street_name),
            display_opt(&expected.address.street_name)
        );
        assert_eq!(
            display_opt(&imported.address.house_number),
            display_opt(&expected.address.house_number)
        );
        assert_eq!(
            display_opt(&imported.address.house_number_addition),
            display_opt(&expected.address.house_number_addition)
        );
        assert_eq!(
            display_opt(&imported.address.postal_code),
            display_opt(&expected.address.postal_code)
        );
        assert_eq!(
            display_opt(&imported.address.locality),
            display_opt(&expected.address.locality)
        );
    }

    #[tokio::test]
    async fn reports_eml_field_errors_by_candidate_position() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let mut list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let result = import_candidate_list_eml(
            &mut list,
            &store,
            invalid_eml().as_bytes(),
            Locale::En,
            "eml210.eml.xml".to_string(),
            0,
        )
        .await;

        match result {
            Err(ImportCandidateListError::Messages(messages)) => {
                assert!(
                    messages
                        .iter()
                        .any(|message| message.contains("position 1")
                            && message.contains("Initials")),
                    "{messages:?}"
                );
                assert!(
                    messages.iter().any(|message| message.contains("position 2")
                        && message.contains("Date of birth")),
                    "{messages:?}"
                );
            }
            other => panic!("expected validation messages, got {other:?}"),
        }

        assert_eq!(store.get_person_count(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn rejects_an_eml_document_that_is_not_a_nomination() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let mut list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let result = import_candidate_list_eml(
            &mut list,
            &store,
            POLLING_STATIONS.as_bytes(),
            Locale::En,
            "eml110b.eml.xml".to_string(),
            0,
        )
        .await;

        match result {
            Err(ImportCandidateListError::Messages(messages)) => {
                assert_eq!(messages.len(), 1);
                assert!(messages[0].contains("not an EML 210"), "{messages:?}");
                assert!(messages[0].contains("110b"), "{messages:?}");
            }
            other => panic!("expected a document type message, got {other:?}"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn rejects_a_file_that_is_not_eml_xml() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let mut list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let result = import_candidate_list_eml(
            &mut list,
            &store,
            b"not xml at all",
            Locale::En,
            "eml210.eml.xml".to_string(),
            0,
        )
        .await;

        match result {
            Err(ImportCandidateListError::Messages(messages)) => {
                assert_eq!(messages.len(), 1);
                assert!(
                    messages[0].contains("The EML file could not be read"),
                    "{messages:?}"
                );
            }
            other => panic!("expected a read error message, got {other:?}"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn rejects_an_eml_file_that_is_not_utf8() -> Result<(), AppError> {
        let store = PgStore::new_for_test();
        let mut list = sample_candidate_list(CandidateListId::new());
        list.create(&store).await?;

        let result = import_candidate_list_eml(
            &mut list,
            &store,
            &[0xff, 0xfe, 0x00],
            Locale::En,
            "eml210.eml.xml".to_string(),
            0,
        )
        .await;

        match result {
            Err(ImportCandidateListError::Messages(messages)) => {
                assert_eq!(messages.len(), 1);
                assert!(messages[0].contains("UTF-8"), "{messages:?}");
            }
            other => panic!("expected an encoding message, got {other:?}"),
        }

        Ok(())
    }

    const NOMINATION: &str = include_str!("testdata/nomination.eml.xml");
    const POLLING_STATIONS: &str = include_str!("testdata/polling_stations.eml.xml");

    /// The first candidate loses its initials, and both candidates get a date
    /// of birth that is not a date.
    fn invalid_eml() -> String {
        NOMINATION.replacen(">H.A.H.A.<", "><", 1).replace(
            "<DateOfBirth>1990-02-01</DateOfBirth>",
            "<DateOfBirth>gisteren</DateOfBirth>",
        )
    }

    const CSV_HEADER: &str = include_str!("testdata/csv_header.csv");

    fn csv_headers() -> &'static str {
        CSV_HEADER.trim_end_matches('\n').trim_end_matches('\r')
    }

    fn valid_csv() -> String {
        format!(
            "{}\r\n{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        )
    }

    fn no_bsn_csv_with_different_first_name() -> String {
        format!(
            "{}\r\n{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        )
    }

    fn mixed_bsn_duplicate_name_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,123456782,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n",
            "H.A.H.A.,Hendrik,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        )
    }

    fn duplicate_no_bsn_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n",
            "H.A.H.A.,Hendrik,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        )
    }

    fn duplicate_bsn_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,123456782,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n",
            "H.A.H.A.,Hendrik,,Jansen,Juinen,NL,123456782,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        )
    }

    fn multiple_invalid_rows_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            ",Henk,,Jansen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n",
            "H.A.H.A.,Henk,,,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,\r\n"
        )
    }
}
