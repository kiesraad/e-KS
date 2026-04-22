//! Filesystem-backed persistence for the event store.
//!
//! Events are stored as length-prefixed postcard frames in a single file per stream.
//! The length prefix lets us read one frame at a time without decoding the whole file,
//! and lets us append new frames at the end. Appends use `O_APPEND` to avoid rewriting.
//!
//! On-disk layout per frame:
//!   [4 bytes: body length (u32, little-endian)]
//!   [body:    postcard-encoded [`Frame`]]
//!
//! [`Frame`] is a versioned enum; postcard encodes the variant discriminant as a
//! varint, giving us a per-frame version marker for future format migrations.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use super::{Store, StoreData, StoreEvent, encryption::EventCipher};
use crate::{AppError, ElectionConfig};

const FRAME_HEADER_LEN: usize = 4;

#[derive(Serialize, Deserialize)]
enum Frame {
    V1 {
        event_id: u64,
        created_at_micros: i64,
        encrypted_payload: Vec<u8>,
    },
}

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

    append_once(
        dir,
        store.stream_id,
        store.election,
        &store.cipher,
        &store_event,
    )
    .await?;

    store.apply_event(next_id, store_event);

    Ok(())
}

pub async fn replay_from_file<D>(store: &Store<D>, dir: &Path) -> Result<usize, AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let path = stream_path(dir, store.stream_id, store.election);
    let mut file = match File::open(&path).await {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(AppError::ServerError(err)),
    };

    let mut last_file_id = 0usize;
    let mut events = Vec::new();

    loop {
        let mut len_buf = [0u8; FRAME_HEADER_LEN];
        match file.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(AppError::ServerError(err)),
        }
        let body_len = u32::from_le_bytes(len_buf) as usize;

        let mut body = vec![0u8; body_len];
        file.read_exact(&mut body)
            .await
            .map_err(AppError::ServerError)?;

        let Frame::V1 {
            event_id,
            created_at_micros,
            encrypted_payload,
        } = postcard::from_bytes::<Frame>(&body)
            .map_err(|e| AppError::EventDecodeError(format!("failed to decode frame: {e}")))?;

        let event_id = event_id as usize;
        let created_at = DateTime::from_timestamp_micros(created_at_micros).unwrap_or_default();

        last_file_id = last_file_id.max(event_id);
        events.push((event_id, created_at, encrypted_payload));
    }

    let mut data = store.data.write();

    for (event_id, created_at, encrypted_payload) in events {
        if data.last_event_id() >= event_id {
            continue;
        }

        let payload = store.cipher.decrypt_owned::<D::Event>(encrypted_payload)?;
        data.apply(StoreEvent {
            event_id,
            payload,
            created_at,
        });
    }

    Ok(last_file_id)
}

/// Check which of the given stream IDs have persisted events on disk (any election).
pub async fn streams_with_data(
    dir: &Path,
    stream_ids: &[uuid::Uuid],
) -> std::collections::HashSet<uuid::Uuid> {
    let wanted: std::collections::HashSet<uuid::Uuid> = stream_ids.iter().copied().collect();
    let mut found = std::collections::HashSet::new();

    visit_non_empty_stream_files(dir, |stream_id, _election| {
        if wanted.contains(&stream_id) {
            found.insert(stream_id);
        }
    })
    .await;

    found
}

/// List the elections that have persisted events under the given stream.
pub async fn elections_for_stream(dir: &Path, stream_id: uuid::Uuid) -> Vec<ElectionConfig> {
    let mut result = Vec::new();
    visit_non_empty_stream_files(dir, |id, election| {
        if id == stream_id {
            result.push(election);
        }
    })
    .await;
    result
}

/// Iterate non-empty `{stream_id}_{election}.bin` files in `dir`, invoking
/// `callback` for each successfully parsed entry. Directory read errors and
/// unparseable filenames are ignored.
async fn visit_non_empty_stream_files(
    dir: &Path,
    mut callback: impl FnMut(uuid::Uuid, ElectionConfig),
) {
    let Ok(mut entries) = fs::read_dir(dir).await else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let Some((stream_id, election)) = parse_stream_filename(&entry.file_name()) else {
            continue;
        };
        if let Ok(meta) = entry.metadata().await
            && meta.len() > 0
        {
            callback(stream_id, election);
        }
    }
}

/// Parse a filename of the form `{uuid}_{election_stable_id}.bin` back into its
/// `(stream_id, election)` parts. Returns `None` for unrelated files.
fn parse_stream_filename(file_name: &std::ffi::OsStr) -> Option<(uuid::Uuid, ElectionConfig)> {
    let name = file_name.to_str()?;
    let stem = name.strip_suffix(".bin")?;
    let (id_str, election_segment) = stem.split_once('_')?;
    let stream_id = uuid::Uuid::parse_str(id_str).ok()?;
    let election = parse_stable_id(election_segment)?;
    Some((stream_id, election))
}

/// Parse the filename-safe form of `stable_id()` (with `:` replaced by `_`,
/// see [`stream_path`]) back into an `ElectionConfig`.
fn parse_stable_id(value: &str) -> Option<ElectionConfig> {
    let (code, region) = match value.split_once('_') {
        Some((code, region)) => (code, Some(region)),
        None => (value, None),
    };
    ElectionConfig::from_code_and_region(code, region)
}

/// Ensure a stream file exists for local storage.
pub async fn ensure_stream_file(
    dir: &Path,
    stream_id: uuid::Uuid,
    election: ElectionConfig,
) -> Result<(), AppError> {
    let path = stream_path(dir, stream_id, election);
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
    election: ElectionConfig,
    cipher: &EventCipher,
    event: &StoreEvent<E>,
) -> Result<(), AppError> {
    let path = stream_path(dir, stream_id, election);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(AppError::ServerError)?;

    let frame = Frame::V1 {
        event_id: event.event_id as u64,
        created_at_micros: event.created_at.timestamp_micros(),
        encrypted_payload: cipher.encrypt(&event.payload)?,
    };

    let body = postcard::to_allocvec(&frame).map_err(|e| {
        AppError::ServerError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    })?;

    let body_len = u32::try_from(body.len()).map_err(|_| {
        AppError::ServerError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame body exceeds u32::MAX",
        ))
    })?;

    let mut buf = Vec::with_capacity(FRAME_HEADER_LEN + body.len());
    buf.extend_from_slice(&body_len.to_le_bytes());
    buf.extend_from_slice(&body);

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

fn stream_path(dir: &Path, stream_id: uuid::Uuid, election: ElectionConfig) -> PathBuf {
    // Election identifiers can contain ':' (e.g. `PS27:GR`); replace with '_'
    // so the filename stays portable on all filesystems.
    let election_segment = election.stable_id().replace(':', "_");
    dir.join(format!("{stream_id}_{election_segment}.bin"))
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

    const TEST_ELECTION: ElectionConfig = ElectionConfig::EK27;

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
            election: TEST_ELECTION,
            persistence: super::super::StorePersistence::None,
            cipher: encryption.derive_cipher(stream_id, TEST_ELECTION),
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

        let path = stream_path(&dir, store.stream_id, TEST_ELECTION);
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
        ensure_stream_file(&dir, stream_id, TEST_ELECTION).await?;

        let path = stream_path(&dir, stream_id, TEST_ELECTION);
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
        append_once(&dir, store.stream_id, TEST_ELECTION, &store.cipher, &first).await?;
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
            election: TEST_ELECTION,
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
            election: TEST_ELECTION,
            persistence: super::super::StorePersistence::None,
            cipher: other_enc.derive_cipher(stream_id, TEST_ELECTION),
            data: Arc::new(RwLock::new(TestData::default())),
        };

        let err = replay_from_file(&wrong_store, &dir)
            .await
            .expect_err("replay must fail with the wrong key");
        assert!(matches!(err, AppError::EventDecodeError(_)));
        assert!(wrong_store.data.read().events.is_empty());

        Ok(())
    }
}
