//! Filesystem-backed persistence for the event store.
//!
//! Events are stored as newline-delimited JSON (one event per line) in a single file
//! per stream. Appends are performed with `O_APPEND` to avoid rewriting the file.

use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};

use super::{Store, StoreData, StoreEvent};
use crate::{AppError, constants::DEFAULT_STREAM_ID};

#[derive(Debug, Serialize, serde::Deserialize)]
struct FileEvent<E> {
    event_id: usize,
    payload: E,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct FileEventRef<'a, E> {
    event_id: usize,
    payload: &'a E,
    created_at: DateTime<Utc>,
}

impl<E> From<StoreEvent<E>> for FileEvent<E> {
    fn from(event: StoreEvent<E>) -> Self {
        Self {
            event_id: event.event_id,
            payload: event.payload,
            created_at: event.created_at,
        }
    }
}

/// Ensure the filesystem storage directory exists.
pub fn init_local(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(AppError::ServerError)
}

/// Load and replay persisted events into the in-memory store.
pub async fn load_from_filesystem<D>(store: &Store<D>, dir: &Path) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    replay_from_file(store, dir).map(|_| ())
}

/// Append the event to the filesystem and apply it to the store.
pub async fn update_in_filesystem<D>(
    store: &Store<D>,
    dir: &Path,
    event: D::Event,
) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    let last_id = catch_up(store, dir)?;
    let next_id = last_id + 1;

    let store_event = StoreEvent {
        event_id: next_id,
        payload: event,
        created_at: Utc::now(),
    };

    append_once(dir, &store_event)?;

    let mut data = store.data.write();

    if data.last_event_id() >= next_id {
        return Ok(());
    }

    data.apply(store_event);
    data.set_last_event_id(next_id);

    Ok(())
}

fn catch_up<D>(store: &Store<D>, dir: &Path) -> Result<usize, AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    replay_from_file(store, dir)
}

fn replay_from_file<D>(store: &Store<D>, dir: &Path) -> Result<usize, AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let path = stream_path(dir);
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(AppError::ServerError(err)),
    };

    let reader = BufReader::new(file);
    let mut data = store.data.write();
    let mut last_file_id = 0usize;

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => return Err(AppError::ServerError(err)),
        };

        if line.trim().is_empty() {
            continue;
        }

        let event: FileEvent<serde_json::Value> = match serde_json::from_str(&line) {
            Ok(event) => event,
            Err(err) => {
                tracing::error!("Failed to deserialize event line: {err:?}");
                continue;
            }
        };

        last_file_id = last_file_id.max(event.event_id);

        if data.last_event_id() >= event.event_id {
            continue;
        }

        match serde_json::from_value::<D::Event>(event.payload) {
            Ok(payload) => {
                let store_event = StoreEvent {
                    event_id: event.event_id,
                    payload,
                    created_at: event.created_at,
                };
                data.apply(store_event);
                data.set_last_event_id(event.event_id);
            }
            Err(err) => {
                tracing::error!("Failed to deserialize event payload: {err:?}");
                continue;
            }
        }
    }

    Ok(last_file_id)
}

fn append_once<E: Serialize>(dir: &Path, event: &StoreEvent<E>) -> Result<(), AppError> {
    let path = stream_path(dir);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(AppError::ServerError)?;

    let mut payload = serde_json::to_vec(&FileEventRef {
        event_id: event.event_id,
        payload: &event.payload,
        created_at: event.created_at,
    })
    .map_err(|_| AppError::InternalServerError)?;

    payload.push(b'\n');

    let written = file.write(&payload).map_err(AppError::ServerError)?;
    if written != payload.len() {
        return Err(AppError::ServerError(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "partial filesystem append",
        )));
    }

    file.sync_data().map_err(AppError::ServerError)?;

    Ok(())
}

fn stream_path(dir: &Path) -> PathBuf {
    dir.join(format!("{DEFAULT_STREAM_ID}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::RwLock;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestEvent {
        label: String,
    }

    #[derive(Default)]
    struct TestData {
        events: Vec<(usize, TestEvent)>,
        last_event_id: usize,
    }

    impl StoreData for TestData {
        type Event = TestEvent;

        fn apply(&mut self, event: StoreEvent<Self::Event>) {
            self.events.push((event.event_id, event.payload));
        }

        fn last_event_id(&self) -> usize {
            self.last_event_id
        }

        fn set_last_event_id(&mut self, event_id: usize) {
            self.last_event_id = event_id;
        }
    }

    fn temp_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("eks-store-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn test_store() -> Store<TestData> {
        Store {
            persistence: super::super::StorePersistence::None,
            data: Arc::new(RwLock::new(TestData::default())),
        }
    }

    #[test]
    fn init_local_creates_directory() {
        let dir = temp_dir().join("nested");
        init_local(&dir).expect("init local");
        assert!(dir.exists());
    }

    #[tokio::test]
    async fn update_and_load_replays_events() -> Result<(), AppError> {
        let dir = temp_dir();
        init_local(&dir)?;

        let store = test_store();
        update_in_filesystem(
            &store,
            &dir,
            TestEvent {
                label: "first".to_string(),
            },
        )
        .await?;
        update_in_filesystem(
            &store,
            &dir,
            TestEvent {
                label: "second".to_string(),
            },
        )
        .await?;

        let path = stream_path(&dir);
        let file_contents = fs::read_to_string(&path).expect("read log");
        assert_eq!(file_contents.lines().count(), 2);

        let fresh = test_store();
        load_from_filesystem(&fresh, &dir).await?;

        let data = fresh.data.read();
        assert_eq!(data.last_event_id(), 2);
        assert_eq!(
            data.events,
            vec![
                (
                    1,
                    TestEvent {
                        label: "first".to_string()
                    }
                ),
                (
                    2,
                    TestEvent {
                        label: "second".to_string()
                    }
                ),
            ]
        );

        Ok(())
    }

    #[tokio::test]
    async fn update_uses_last_event_id_from_file() -> Result<(), AppError> {
        let dir = temp_dir();
        init_local(&dir)?;

        let first = StoreEvent::new_at(
            5,
            TestEvent {
                label: "existing".to_string(),
            },
            Utc::now(),
        );
        append_once(&dir, &first)?;

        let store = test_store();
        update_in_filesystem(
            &store,
            &dir,
            TestEvent {
                label: "next".to_string(),
            },
        )
        .await?;

        let file_contents = fs::read_to_string(stream_path(&dir)).expect("read log");
        let last_line = file_contents.lines().last().expect("last line");
        let event: FileEvent<TestEvent> =
            serde_json::from_str(last_line).expect("parse last event");
        assert_eq!(event.event_id, 6);

        Ok(())
    }

    #[tokio::test]
    async fn load_skips_invalid_lines() -> Result<(), AppError> {
        let dir = temp_dir();
        init_local(&dir)?;
        let path = stream_path(&dir);

        let valid = serde_json::to_string(&FileEventRef {
            event_id: 1,
            payload: &TestEvent {
                label: "ok".to_string(),
            },
            created_at: Utc::now(),
        })
        .expect("serialize event");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("open log");
        writeln!(file, "{valid}").expect("write valid");
        writeln!(file, "not json").expect("write invalid");

        let store = test_store();
        load_from_filesystem(&store, &dir).await?;

        let data = store.data.read();
        assert_eq!(data.last_event_id(), 1);
        assert_eq!(
            data.events,
            vec![(
                1,
                TestEvent {
                    label: "ok".to_string()
                }
            )]
        );

        Ok(())
    }
}
