use std::fmt::Display;

use axum::{
    body::Body,
    http::{HeaderValue, Response},
    response::IntoResponse,
};
use csv::{Reader, Writer};
use serde::{Serialize, de::DeserializeOwned};

use crate::{AppError, Locale, OptionStringExt, trans, utils::no_cache_headers};

pub enum CsvError {
    FormatError {
        candidate_number: usize,
        message: csv::ErrorKind,
    },
    ParseError {
        candidate_number: usize,
        field_name: String,
        message: String,
    },
}

impl Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message(Locale::default()))
    }
}

impl CsvError {
    pub fn message(&self, locale: Locale) -> String {
        match self {
            CsvError::FormatError {
                candidate_number,
                message,
            } => trans!(
                "candidate_list.import_errors.format_error",
                locale,
                candidate_number,
                format_error_kind(message, locale)
            ),
            CsvError::ParseError {
                candidate_number,
                field_name,
                message,
            } => trans!(
                "candidate_list.import_errors.parse_error",
                locale,
                candidate_number,
                translated_field_name(field_name, locale),
                message
            ),
        }
    }
}

fn translated_field_name(field_name: &str, locale: Locale) -> String {
    let field_key = format!(
        "person.fields.{}",
        field_name.rsplit('.').next().unwrap_or(field_name)
    );
    let translated = match locale {
        crate::Locale::En => crate::translate::LOCALE_EN.get(&field_key),
        crate::Locale::Nl => crate::translate::LOCALE_NL.get(&field_key),
    }
    .to_string_or_default();

    if translated.is_empty() {
        field_name.to_string()
    } else {
        translated
    }
}

fn format_error_kind(kind: &csv::ErrorKind, locale: Locale) -> String {
    match kind {
        csv::ErrorKind::Io(err) => trans!("candidate_list.import_errors.csv.io", locale, err),
        csv::ErrorKind::Utf8 { pos: None, err } => trans!(
            "candidate_list.import_errors.csv.utf8",
            locale,
            err.field(),
            err
        ),
        csv::ErrorKind::Utf8 {
            pos: Some(pos),
            err,
        } => trans!(
            "candidate_list.import_errors.csv.utf8_with_position",
            locale,
            pos.record(),
            pos.line(),
            err.field(),
            pos.byte(),
            err
        ),
        csv::ErrorKind::UnequalLengths {
            pos: None,
            expected_len,
            len,
        } => trans!(
            "candidate_list.import_errors.csv.unequal_lengths",
            locale,
            len,
            expected_len
        ),
        csv::ErrorKind::UnequalLengths {
            pos: Some(pos),
            expected_len,
            len,
        } => trans!(
            "candidate_list.import_errors.csv.unequal_lengths_with_position",
            locale,
            pos.record(),
            pos.line(),
            pos.byte(),
            len,
            expected_len
        ),
        csv::ErrorKind::Seek => trans!("candidate_list.import_errors.csv.seek", locale),
        csv::ErrorKind::Serialize(err) => {
            trans!("candidate_list.import_errors.csv.serialize", locale, err)
        }
        csv::ErrorKind::Deserialize { pos: None, err } => {
            trans!("candidate_list.import_errors.csv.deserialize", locale, err)
        }
        csv::ErrorKind::Deserialize {
            pos: Some(pos),
            err,
        } => trans!(
            "candidate_list.import_errors.csv.deserialize_with_position",
            locale,
            pos.record(),
            pos.line(),
            pos.byte(),
            err
        ),
        _ => trans!(
            "candidate_list.import_errors.csv.unknown",
            locale,
            format!("{kind:?}")
        ),
    }
}

pub struct Csv<T: Serialize + DeserializeOwned> {
    pub records: Vec<T>,
    pub filename: String,
}

impl<T: Serialize + DeserializeOwned> Csv<T> {
    pub fn generate_csv_response(&self) -> Result<Response<Body>, AppError> {
        let mut csv_writer = Writer::from_writer(vec![]);
        for record in &self.records {
            if csv_writer.serialize(record).is_err() {
                return Err(AppError::InternalServerError);
            }
        }
        let data = if let Ok(data) = csv_writer.into_inner() {
            data
        } else {
            return Err(AppError::InternalServerError);
        };
        let headers = no_cache_headers::generate_attachment_headers(
            self.filename.as_str(),
            HeaderValue::from_static("text/csv"),
        )?;

        Ok((headers, data).into_response())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Vec<T>, Vec<CsvError>> {
        let mut records = vec![];
        let mut errors = vec![];

        Reader::from_reader(data)
            .deserialize::<T>()
            .enumerate()
            .for_each(|(count, res)| match res {
                Ok(record) => records.push(record),
                Err(error) => errors.push(CsvError::FormatError {
                    candidate_number: count + 1,
                    message: error.into_kind(),
                }),
            });
        if errors.is_empty() {
            Ok(records)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestRecord {
        #[allow(dead_code)]
        value: u32,
    }

    fn position(record: u64, line: u64, byte: u64) -> csv::Position {
        let mut pos = csv::Position::new();
        pos.set_record(record).set_line(line).set_byte(byte);
        pos
    }

    fn deserialize_error() -> csv::DeserializeError {
        let mut reader = csv::Reader::from_reader("value\nabc\n".as_bytes());
        let error = reader
            .deserialize::<TestRecord>()
            .next()
            .expect("expected one row")
            .expect_err("expected deserialization error");

        match error.into_kind() {
            csv::ErrorKind::Deserialize { err, .. } => err,
            other => panic!("expected deserialize error, got {other:?}"),
        }
    }

    fn utf8_error() -> csv::Utf8Error {
        let byte_record = csv::ByteRecord::from(vec![&b"quux"[..], &b"foo\xFFbar"[..], &b"c"[..]]);
        let error = csv::StringRecord::from_byte_record(byte_record)
            .expect_err("expected utf-8 validation error");

        error.utf8_error().clone()
    }

    #[test]
    fn formats_error_kind_messages_in_english() {
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Io(std::io::Error::other("disk failed")),
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: the file could not be read. disk failed"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Utf8 {
                    pos: None,
                    err: utf8_error(),
                },
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: field 1 contains unreadable text. Please save the file as UTF-8 and try again. invalid utf-8: invalid UTF-8 in field 1 near byte index 3"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Utf8 {
                    pos: Some(position(2, 4, 18)),
                    err: utf8_error(),
                },
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: record 2 on line 4 contains unreadable text in field 1 near character 18. Please save the file as UTF-8 and try again. invalid utf-8: invalid UTF-8 in field 1 near byte index 3"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::UnequalLengths {
                    pos: None,
                    expected_len: 4,
                    len: 2,
                },
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: this row has 2 columns, but earlier rows have 4. Please make sure each row has the same number of columns."
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::UnequalLengths {
                    pos: Some(position(2, 4, 18)),
                    expected_len: 4,
                    len: 2,
                },
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: record 2 on line 4 near character 18 has 2 columns, but earlier rows have 4. Please make sure each row has the same number of columns."
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Seek,
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: the file could not be read correctly. Please export the CSV again and try again."
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Serialize("unsupported value".to_string()),
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: the file contains a value that could not be processed. unsupported value"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Deserialize {
                    pos: None,
                    err: deserialize_error(),
                },
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: one of the values in this row is in the wrong format. field 0: invalid digit found in string"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Deserialize {
                    pos: Some(position(1, 2, 6)),
                    err: deserialize_error(),
                },
            }
            .message(Locale::En),
            "The candidate on line 3 could not be imported: record 1 on line 2 near character 6 contains a value in the wrong format. field 0: invalid digit found in string"
        );
        assert_eq!(
            CsvError::ParseError {
                candidate_number: 4,
                field_name: "postal_code".to_string(),
                message: "invalid value".to_string(),
            }
            .message(Locale::En),
            "The candidate on line 4 could not be imported. Please check field \"Postal code\": invalid value"
        );
    }

    #[test]
    fn formats_error_kind_messages_in_dutch() {
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Io(std::io::Error::other("disk failed")),
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: het bestand kon niet worden gelezen. disk failed"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Utf8 {
                    pos: None,
                    err: utf8_error(),
                },
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: veld 1 bevat onleesbare tekst. Sla het bestand op als UTF-8 en probeer het opnieuw. invalid utf-8: invalid UTF-8 in field 1 near byte index 3"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Utf8 {
                    pos: Some(position(2, 4, 18)),
                    err: utf8_error(),
                },
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: record 2 op regel 4 bevat onleesbare tekst in veld 1 rond teken 18. Sla het bestand op als UTF-8 en probeer het opnieuw. invalid utf-8: invalid UTF-8 in field 1 near byte index 3"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::UnequalLengths {
                    pos: None,
                    expected_len: 4,
                    len: 2,
                },
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: deze rij heeft 2 kolommen, maar eerdere rijen hebben er 4. Controleer of elke rij evenveel kolommen heeft."
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::UnequalLengths {
                    pos: Some(position(2, 4, 18)),
                    expected_len: 4,
                    len: 2,
                },
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: record 2 op regel 4 rond teken 18 heeft 2 kolommen, maar eerdere rijen hebben er 4. Controleer of elke rij evenveel kolommen heeft."
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Seek,
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: het bestand kon niet goed worden gelezen. Exporteer het CSV-bestand opnieuw en probeer het daarna nog eens."
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Serialize("unsupported value".to_string()),
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: het bestand bevat een waarde die niet verwerkt kon worden. unsupported value"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Deserialize {
                    pos: None,
                    err: deserialize_error(),
                },
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: een van de waarden in deze rij heeft niet het juiste formaat. field 0: invalid digit found in string"
        );
        assert_eq!(
            CsvError::FormatError {
                candidate_number: 3,
                message: csv::ErrorKind::Deserialize {
                    pos: Some(position(1, 2, 6)),
                    err: deserialize_error(),
                },
            }
            .message(Locale::Nl),
            "De kandidaat op regel 3 kon niet worden geïmporteerd: record 1 op regel 2 rond teken 6 bevat een waarde met een onjuist formaat. field 0: invalid digit found in string"
        );
        assert_eq!(
            CsvError::ParseError {
                candidate_number: 4,
                field_name: "postal_code".to_string(),
                message: "invalid value".to_string(),
            }
            .message(Locale::Nl),
            "De kandidaat op regel 4 kon niet worden geïmporteerd. Controleer veld \"Postcode\": invalid value"
        );
    }

    #[test]
    fn display_uses_default_locale() {
        let message = CsvError::FormatError {
            candidate_number: 1,
            message: csv::ErrorKind::Serialize("invalid record".to_string()),
        }
        .to_string();

        assert_eq!(
            message,
            "De kandidaat op regel 1 kon niet worden geïmporteerd: het bestand bevat een waarde die niet verwerkt kon worden. invalid record"
        );
    }
}
