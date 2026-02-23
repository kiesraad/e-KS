//! Filesystem-backed persistence for the event store.
//!
//! Events are stored as newline-delimited JSON (one event per line) in a single file
//! per stream. Appends are performed with `O_APPEND` to avoid rewriting the file.

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

use super::{Store, StoreData, StoreEvent};
use crate::{AppError, constants::DEFAULT_STREAM_ID};

/// Ensure the filesystem storage directory exists.
pub async fn init_local(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).await.map_err(AppError::ServerError)
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
    let last_id = replay_from_file(store, dir).await?;
    let next_id = last_id + 1;

    let store_event = StoreEvent {
        event_id: next_id,
        payload: event,
        created_at: Utc::now(),
    };

    append_once(dir, &store_event).await?;

    store.apply_event(next_id, store_event);

    Ok(())
}

pub async fn replay_from_file<D>(store: &Store<D>, dir: &Path) -> Result<usize, AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let path = stream_path(dir);
    let file = match File::open(&path).await {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(AppError::ServerError(err)),
    };

    let reader = BufReader::new(file);
    let mut last_file_id = 0usize;
    let mut events = Vec::new();

    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }

        let event: StoreEvent<serde_json::Value> = match serde_json::from_str(&line) {
            Ok(event) => event,
            Err(err) => {
                tracing::error!("Failed to deserialize event line: {err:?}");
                continue;
            }
        };

        last_file_id = last_file_id.max(event.event_id);
        events.push(event);
    }

    let mut data = store.data.write();

    for event in events {
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

async fn append_once<E: Serialize>(dir: &Path, event: &StoreEvent<E>) -> Result<(), AppError> {
    let path = stream_path(dir);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(AppError::ServerError)?;

    let mut payload = serde_json::to_vec(&StoreEvent {
        event_id: event.event_id,
        payload: &event.payload,
        created_at: event.created_at,
    })
    .map_err(|e| AppError::ServerError(e.into()))?;

    payload.push(b'\n');

    let written = file.write(&payload).await.map_err(AppError::ServerError)?;
    if written != payload.len() {
        return Err(AppError::ServerError(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "partial filesystem append",
        )));
    }

    file.sync_data().await.map_err(AppError::ServerError)?;

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

    async fn temp_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("eks-store-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).await.expect("create temp dir");
        dir
    }

    fn test_store() -> Store<TestData> {
        Store {
            persistence: super::super::StorePersistence::None,
            data: Arc::new(RwLock::new(TestData::default())),
        }
    }

    #[tokio::test]
    async fn init_local_creates_directory() {
        let dir = temp_dir().await.join("nested");
        init_local(&dir).await.expect("init local");
        assert!(dir.exists());
    }

    #[tokio::test]
    async fn update_and_load_replays_events() -> Result<(), AppError> {
        let dir = temp_dir().await;
        init_local(&dir).await?;

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
        let file_contents = fs::read_to_string(&path).await.expect("read log");
        assert_eq!(file_contents.lines().count(), 2);

        let fresh = test_store();
        replay_from_file(&fresh, &dir).await?;

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
        let dir = temp_dir().await;
        init_local(&dir).await?;

        let first = StoreEvent::new_at(
            5,
            TestEvent {
                label: "existing".to_string(),
            },
            Utc::now(),
        );
        append_once(&dir, &first).await?;

        let store = test_store();
        update_in_filesystem(
            &store,
            &dir,
            TestEvent {
                label: "next".to_string(),
            },
        )
        .await?;

        let file_contents = fs::read_to_string(stream_path(&dir))
            .await
            .expect("read log");
        let last_line = file_contents.lines().last().expect("last line");
        let event: StoreEvent<TestEvent> =
            serde_json::from_str(last_line).expect("parse last event");
        assert_eq!(event.event_id, 6);

        Ok(())
    }

    #[tokio::test]
    async fn load_skips_invalid_lines() -> Result<(), AppError> {
        let dir = temp_dir().await;
        init_local(&dir).await?;
        let path = stream_path(&dir);

        let valid = serde_json::to_string(&StoreEvent {
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
            .await
            .expect("open log");

        file.write_all(valid.as_bytes()).await.expect("write valid");
        file.write_all(b"\n").await.expect("write valid");
        file.write_all(b"not json\n").await.expect("write invalid");

        let store = test_store();
        replay_from_file(&store, &dir).await?;

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
