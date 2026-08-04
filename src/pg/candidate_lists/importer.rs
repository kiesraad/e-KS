use crate::{
    AppError, Locale, MAX_CANDIDATES, PgEvent, PgStore,
    candidate_lists::{CSV_HEADERS, CandidateRecord, CandidateRecordCsv},
    core::{Csv, CsvError},
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
    let persons = collect_persons(records, store.get_persons(), locale)?;
    emit_import_event(list, store, persons, file_name, file_size).await
}

/// Information about a successful import that the caller surfaces to the user.
#[derive(Debug)]
pub(crate) struct ImportOutcome {
    /// The number of candidates in the file exceeded [`MAX_CANDIDATES`] and the
    /// list was truncated to the maximum.
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

fn csv_error_messages(errors: Vec<CsvError>, locale: Locale) -> Vec<String> {
    errors
        .into_iter()
        .map(|error| error.message(locale))
        .collect()
}

fn collect_persons(
    records: Vec<CandidateRecord>,
    existing_persons: Vec<Person>,
    locale: Locale,
) -> Result<Vec<PreparedPerson>, ImportCandidateListError> {
    let mut prepared_people = Vec::new();
    let mut errors = Vec::new();

    for (index, record) in records.into_iter().enumerate() {
        let candidate_number = index + 1;
        match validate_record(record, candidate_number, locale) {
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
    candidate_number: usize,
    locale: Locale,
) -> Result<Person, ImportCandidateListError> {
    record
        .validate_create()
        .map(refresh_bag_checks)
        .map_err(|error| {
            ImportCandidateListError::Messages(field_error_messages(
                candidate_number,
                error.errors(),
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
    candidate_number: usize,
    errors: FieldErrors,
    locale: Locale,
) -> Vec<String> {
    errors
        .into_iter()
        .map(|(field_name, error)| {
            CsvError::ParseError {
                candidate_number,
                field_name,
                message: error.message(locale),
            }
            .message(locale)
        })
        .collect()
}

async fn emit_import_event(
    list: &mut CandidateList,
    store: &PgStore,
    persons: Vec<PreparedPerson>,
    file_name: String,
    file_size: usize,
) -> Result<ImportOutcome, ImportCandidateListError> {
    let mut candidates = persons.iter().map(|p| p.person.id).collect::<Vec<_>>();
    let capped = candidates.len() > MAX_CANDIDATES;
    candidates.truncate(MAX_CANDIDATES);

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

    Ok(ImportOutcome { capped })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
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
                .map(|value| value.to_string()),
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
                .map(|value| value.to_string()),
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
                .map(|value| value.to_string()),
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
                "H.A.H.A.,Henk,,Jansen{index},Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
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
            "H.A.H.A.,Henk,,Jansen,Amsterdam,NL,kandidaat heeft geen BSN,01-02-1990,v,1012JS,1,,Dam,Amsterdam,,,,,,,,,\r\n",
            "H.A.H.A.,Piet,,Pietersen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
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
            "H.A.H.A.,Henk,,Jansen,Antwerp,BE,kandidaat heeft geen BSN,01-02-1990,v,,,,,,P.,Pietje,,Puk,1012JS,1,,Dam,Amsterdam\r\n"
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
                assert!(messages.iter().any(|message| message.contains("line 1")));
                assert!(messages.iter().any(|message| message.contains("line 2")));
            }
            other => panic!("expected validation messages, got {other:?}"),
        }

        assert_eq!(store.get_person_count(), 0);

        Ok(())
    }

    const CSV_HEADER: &str = include_str!("testdata/csv_header.csv");

    fn csv_headers() -> &'static str {
        CSV_HEADER.trim_end_matches('\n').trim_end_matches('\r')
    }

    fn valid_csv() -> String {
        format!(
            "{}\r\n{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
        )
    }

    fn no_bsn_csv_with_different_first_name() -> String {
        format!(
            "{}\r\n{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
        )
    }

    fn mixed_bsn_duplicate_name_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,123456782,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n",
            "H.A.H.A.,Hendrik,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
        )
    }

    fn duplicate_no_bsn_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n",
            "H.A.H.A.,Hendrik,,Jansen,Juinen,NL,,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
        )
    }

    fn duplicate_bsn_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            "H.A.H.A.,Henk,,Jansen,Juinen,NL,123456782,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n",
            "H.A.H.A.,Hendrik,,Jansen,Juinen,NL,123456782,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
        )
    }

    fn multiple_invalid_rows_csv() -> String {
        format!(
            "{}\r\n{}{}",
            csv_headers(),
            ",Henk,,Jansen,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n",
            "H.A.H.A.,Henk,,,Juinen,NL,kandidaat heeft geen BSN,01-02-1990,v,1234AB,10,A,Stationsstraat,Juinen,,,,,,,,,\r\n"
        )
    }
}
