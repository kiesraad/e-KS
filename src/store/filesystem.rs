//! Filesystem-backed persistence for the event store.
//!
//! Events are stored as length-prefixed binary frames in a single file per stream.
//! Each frame contains an event_id, created_at timestamp, and an encrypted postcard
//! payload. Appends are performed with `O_APPEND` to avoid rewriting the file.
//!
//! Frame format (all integers little-endian):
//!   [4 bytes: frame length (u32, excludes this header)]
//!   [8 bytes: event_id (u64)]
//!   [8 bytes: created_at (i64, Unix timestamp in microseconds)]
//!   [remaining: encrypted payload (nonce || ciphertext || tag)]

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use super::{Store, StoreData, StoreEvent, encryption::EventCipher};
use crate::AppError;

const EVENT_ID_LEN: usize = 8;
const CREATED_AT_LEN: usize = 8;
const FRAME_HEADER_LEN: usize = 4;
const METADATA_LEN: usize = EVENT_ID_LEN + CREATED_AT_LEN;

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

    append_once(dir, store.stream_id, &store.cipher, &store_event).await?;

    store.apply_event(next_id, store_event);

    Ok(())
}

pub async fn replay_from_file<D>(store: &Store<D>, dir: &Path) -> Result<usize, AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let path = stream_path(dir, store.stream_id);
    let mut file = match File::open(&path).await {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(AppError::ServerError(err)),
    };

    let mut last_file_id = 0usize;
    let mut events = Vec::new();

    loop {
        // Read frame length (4 bytes LE)
        let mut len_buf = [0u8; FRAME_HEADER_LEN];
        match file.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(AppError::ServerError(err)),
        }
        let frame_len = u32::from_le_bytes(len_buf) as usize;

        if frame_len < METADATA_LEN {
            tracing::error!("Frame too short ({frame_len} bytes), skipping rest of file");
            break;
        }

        // Read full frame
        let mut frame = vec![0u8; frame_len];
        if let Err(err) = file.read_exact(&mut frame).await {
            tracing::error!("Failed to read frame body: {err:?}");
            break;
        }

        // Parse metadata
        let event_id = u64::from_le_bytes(frame[..EVENT_ID_LEN].try_into().unwrap()) as usize;
        let created_at_micros =
            i64::from_le_bytes(frame[EVENT_ID_LEN..METADATA_LEN].try_into().unwrap());
        let encrypted_payload = &frame[METADATA_LEN..];

        let created_at = DateTime::from_timestamp_micros(created_at_micros).unwrap_or_default();

        last_file_id = last_file_id.max(event_id);
        events.push((event_id, created_at, encrypted_payload.to_vec()));
    }

    let mut data = store.data.write();

    for (event_id, created_at, encrypted_payload) in events {
        if data.last_event_id() >= event_id {
            continue;
        }

        match store.cipher.decrypt_owned::<D::Event>(encrypted_payload) {
            Ok(payload) => {
                let store_event = StoreEvent {
                    event_id,
                    payload,
                    created_at,
                };
                data.apply(store_event);
            }
            Err(err) => {
                tracing::error!("Failed to decrypt/deserialize event {event_id}: {err:?}");
                continue;
            }
        }
    }

    Ok(last_file_id)
}

/// Check which of the given stream IDs have persisted events on disk.
pub async fn streams_with_data(
    dir: &Path,
    stream_ids: &[uuid::Uuid],
) -> std::collections::HashSet<uuid::Uuid> {
    let mut result = std::collections::HashSet::new();
    for &id in stream_ids {
        let path = stream_path(dir, id);
        if let Ok(meta) = fs::metadata(&path).await
            && meta.len() > 0
        {
            result.insert(id);
        }
    }
    result
}

/// Ensure a stream file exists for local storage.
pub async fn ensure_stream_file(dir: &Path, stream_id: uuid::Uuid) -> Result<(), AppError> {
    let path = stream_path(dir, stream_id);
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .await
        .map_err(AppError::ServerError)?;

    Ok(())
}

async fn append_once<E: Serialize>(
    dir: &Path,
    stream_id: uuid::Uuid,
    cipher: &EventCipher,
    event: &StoreEvent<E>,
) -> Result<(), AppError> {
    let path = stream_path(dir, stream_id);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(AppError::ServerError)?;

    let encrypted_payload = cipher.encrypt(&event.payload)?;

    let created_at_micros = event.created_at.timestamp_micros();
    let frame_len = (METADATA_LEN + encrypted_payload.len()) as u32;

    let mut buf = Vec::with_capacity(FRAME_HEADER_LEN + frame_len as usize);
    buf.extend_from_slice(&frame_len.to_le_bytes());
    buf.extend_from_slice(&(event.event_id as u64).to_le_bytes());
    buf.extend_from_slice(&created_at_micros.to_le_bytes());
    buf.extend_from_slice(&encrypted_payload);

    let written = file.write(&buf).await.map_err(AppError::ServerError)?;
    if written != buf.len() {
        return Err(AppError::ServerError(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "partial filesystem append",
        )));
    }

    file.sync_data().await.map_err(AppError::ServerError)?;

    Ok(())
}

fn stream_path(dir: &Path, stream_id: uuid::Uuid) -> PathBuf {
    dir.join(format!("{stream_id}.bin"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::encryption::EventEncryption;
    use parking_lot::RwLock;
    use secrecy::SecretString;
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
        type Init = ();

        fn new(_: ()) -> Self {
            Self::default()
        }

        fn apply(&mut self, event: StoreEvent<Self::Event>) {
            self.last_event_id = event.event_id;
            self.events.push((event.event_id, event.payload));
        }

        fn last_event_id(&self) -> usize {
            self.last_event_id
        }
    }

    fn test_encryption() -> EventEncryption {
        EventEncryption::new(&SecretString::from("test-secret"))
    }

    async fn temp_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("eks-store-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).await.expect("create temp dir");
        dir
    }

    fn test_store(stream_id: uuid::Uuid) -> Store<TestData> {
        let encryption = test_encryption();
        Store {
            stream_id,
            persistence: super::super::StorePersistence::None,
            cipher: encryption.derive_cipher(stream_id),
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

        let stream_id = uuid::Uuid::new_v4();
        let store = test_store(stream_id);
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

        let path = stream_path(&dir, store.stream_id);
        let file_contents = fs::read(&path).await.expect("read log");
        // Binary file should not be empty
        assert!(!file_contents.is_empty());

        let fresh = test_store(stream_id);
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
    async fn ensure_stream_creates_empty_file() -> Result<(), AppError> {
        let dir = temp_dir().await;
        init_local(&dir).await?;

        let stream_id = uuid::Uuid::new_v4();
        ensure_stream_file(&dir, stream_id).await?;

        let path = stream_path(&dir, stream_id);
        let metadata = fs::metadata(&path).await.map_err(AppError::ServerError)?;
        assert_eq!(metadata.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn update_uses_last_event_id_from_file() -> Result<(), AppError> {
        let dir = temp_dir().await;
        init_local(&dir).await?;

        let store = test_store(uuid::Uuid::new_v4());
        let first = StoreEvent::new_at(
            5,
            TestEvent {
                label: "existing".to_string(),
            },
            Utc::now(),
        );
        append_once(&dir, store.stream_id, &store.cipher, &first).await?;
        update_in_filesystem(
            &store,
            &dir,
            TestEvent {
                label: "next".to_string(),
            },
        )
        .await?;

        // Replay and check that the new event got ID 6
        // Need to use the same encryption key, which test_store does by using same stream_id
        // But test_store creates a new encryption instance - same secret though, so same key.
        let fresh = Store {
            stream_id: store.stream_id,
            persistence: super::super::StorePersistence::None,
            cipher: store.cipher.clone(),
            data: Arc::new(RwLock::new(TestData::default())),
        };
        replay_from_file(&fresh, &dir).await?;

        let data = fresh.data.read();
        assert_eq!(data.last_event_id(), 6);
        assert_eq!(data.events.len(), 2);
        assert_eq!(data.events[1].0, 6);

        Ok(())
    }

    #[tokio::test]
    async fn different_key_cannot_read_events() -> Result<(), AppError> {
        let dir = temp_dir().await;
        init_local(&dir).await?;

        let stream_id = uuid::Uuid::new_v4();
        let store = test_store(stream_id);
        update_in_filesystem(
            &store,
            &dir,
            TestEvent {
                label: "secret".to_string(),
            },
        )
        .await?;

        // Create a store with a different encryption key
        let other_enc = EventEncryption::new(&SecretString::from("wrong-secret"));
        let wrong_store = Store {
            stream_id,
            persistence: super::super::StorePersistence::None,
            cipher: other_enc.derive_cipher(stream_id),
            data: Arc::new(RwLock::new(TestData::default())),
        };

        // Replay should fail to decrypt (events are skipped)
        replay_from_file(&wrong_store, &dir).await?;
        let data = wrong_store.data.read();
        assert_eq!(data.last_event_id(), 0);
        assert!(data.events.is_empty());

        Ok(())
    }
}
