use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use limen_domain::GameId;
use limen_session::SessionEvent;
use serde::{Deserialize, Serialize};

const JOURNAL_FILE: &str = "session-events-v1.jsonl";
const SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredSessionEvent {
    schema_version: u16,
    pub game_id: GameId,
    pub event: SessionEvent,
}

impl StoredSessionEvent {
    pub fn new(game_id: GameId, event: SessionEvent) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            game_id,
            event,
        }
    }

    fn validate(&self) -> Result<(), PersistenceError> {
        if self.schema_version == SCHEMA_VERSION {
            Ok(())
        } else {
            Err(PersistenceError::UnsupportedSchemaVersion)
        }
    }
}

pub trait SessionEventStore: Send + Sync {
    fn load(&self) -> Result<Vec<StoredSessionEvent>, PersistenceError>;
    fn append(&self, event: &StoredSessionEvent) -> Result<(), PersistenceError>;
}

#[derive(Clone, Debug)]
pub struct FileSessionEventStore {
    journal: PathBuf,
    access: Arc<Mutex<()>>,
}

impl FileSessionEventStore {
    pub fn create(root: &Path) -> Result<Self, PersistenceError> {
        validate_root(root)?;
        fs::create_dir_all(root)?;
        let canonical_root = fs::canonicalize(root)?;
        let journal = canonical_root.join(JOURNAL_FILE);
        if journal.parent() != Some(canonical_root.as_path()) {
            return Err(PersistenceError::UnsafeJournalPath);
        }
        reject_symlink(&journal)?;

        Ok(Self {
            journal,
            access: Arc::new(Mutex::new(())),
        })
    }

    pub fn journal_path(&self) -> &Path {
        &self.journal
    }

    fn lock(&self) -> Result<MutexGuard<'_, ()>, PersistenceError> {
        self.access
            .lock()
            .map_err(|_| PersistenceError::LockPoisoned)
    }
}

impl SessionEventStore for FileSessionEventStore {
    fn load(&self) -> Result<Vec<StoredSessionEvent>, PersistenceError> {
        let _access = self.lock()?;
        reject_symlink(&self.journal)?;
        let bytes = match fs::read(&self.journal) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        if bytes.is_empty() {
            return Ok(Vec::new());
        }
        if bytes.last() != Some(&b'\n') {
            return Err(PersistenceError::TruncatedJournal);
        }

        let mut events = Vec::new();
        for line in bytes.split(|byte| *byte == b'\n') {
            if line.is_empty() {
                continue;
            }
            let event: StoredSessionEvent = serde_json::from_slice(line)?;
            event.validate()?;
            events.push(event);
        }
        Ok(events)
    }

    fn append(&self, event: &StoredSessionEvent) -> Result<(), PersistenceError> {
        event.validate()?;
        let _access = self.lock()?;
        reject_symlink(&self.journal)?;
        let mut encoded = serde_json::to_vec(event)?;
        encoded.push(b'\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.journal)?;
        file.write_all(&encoded)?;
        file.flush()?;
        file.sync_data()?;
        Ok(())
    }
}

fn validate_root(root: &Path) -> Result<(), PersistenceError> {
    if !root.is_absolute()
        || root
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(PersistenceError::InvalidRoot);
    }
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), PersistenceError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(PersistenceError::UnsafeJournalPath)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[derive(Debug)]
pub enum PersistenceError {
    InvalidRoot,
    UnsafeJournalPath,
    UnsupportedSchemaVersion,
    TruncatedJournal,
    LockPoisoned,
    Io(io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot => formatter.write_str("persistence root must be an absolute path"),
            Self::UnsafeJournalPath => {
                formatter.write_str("session journal is outside its authorized root")
            }
            Self::UnsupportedSchemaVersion => {
                formatter.write_str("session journal uses an unsupported schema version")
            }
            Self::TruncatedJournal => formatter.write_str("session journal is truncated"),
            Self::LockPoisoned => formatter.write_str("session journal lock is unavailable"),
            Self::Io(_) => formatter.write_str("session journal could not be accessed"),
            Self::Json(_) => formatter.write_str("session journal contains invalid JSON"),
        }
    }
}

impl Error for PersistenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for PersistenceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use limen_domain::{SessionId, SessionState};

    use super::*;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn create() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "limen-persistence-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let safe_name = self
                .0
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("limen-persistence-test-"));
            if safe_name {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }

    fn stored_event() -> StoredSessionEvent {
        StoredSessionEvent::new(
            GameId::parse("game-placeholder-001").unwrap(),
            SessionEvent {
                sequence: 1,
                session_id: SessionId::parse("session-simulated-000001").unwrap(),
                previous: SessionState::Requested,
                current: SessionState::Validating,
                outcome: None,
            },
        )
    }

    #[test]
    fn journal_round_trip_is_versioned_and_newline_delimited() {
        let directory = TestDirectory::create();
        let store = FileSessionEventStore::create(&directory.0).unwrap();
        let event = stored_event();

        store.append(&event).unwrap();

        assert_eq!(store.load().unwrap(), vec![event]);
        assert!(fs::read(store.journal_path()).unwrap().ends_with(b"\n"));
    }

    #[test]
    fn relative_root_and_truncated_journal_fail_closed() {
        assert!(matches!(
            FileSessionEventStore::create(Path::new("relative-data")),
            Err(PersistenceError::InvalidRoot)
        ));

        let directory = TestDirectory::create();
        let store = FileSessionEventStore::create(&directory.0).unwrap();
        fs::write(store.journal_path(), b"{\"schema_version\":1").unwrap();
        assert!(matches!(
            store.load(),
            Err(PersistenceError::TruncatedJournal)
        ));
    }
}
