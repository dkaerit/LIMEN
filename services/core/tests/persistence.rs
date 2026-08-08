use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use limen_bridge_fake::{FakeBridge, FakeRuntimeMode};
use limen_core::{
    Core, CoreError, FileSessionEventStore, SessionEventStore, StoredSessionEvent,
};
use limen_domain::{GameId, SessionId, SessionState};
use limen_session::SessionEvent;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn create() -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "limen-core-persistence-test-{}-{sequence}",
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
            .is_some_and(|name| name.starts_with("limen-core-persistence-test-"));
        if safe_name {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn fake_runtime() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_limen-fake-runtime"))
}

fn game_id() -> GameId {
    GameId::parse("game-placeholder-001").unwrap()
}

#[test]
fn completed_session_is_rebuilt_and_sequence_continues_after_restart() {
    let directory = TestDirectory::create();
    let store = Arc::new(FileSessionEventStore::create(&directory.0).unwrap());
    let core = Core::with_event_store(store.clone()).unwrap();
    let bridge = FakeBridge::new(fake_runtime(), FakeRuntimeMode::Normal);
    let first = core
        .run_session(
            &bridge,
            game_id(),
            Duration::from_secs(2),
            &AtomicBool::new(false),
        )
        .unwrap();
    drop(core);

    let restored = Core::with_event_store(store).unwrap();
    assert_eq!(
        restored.home_snapshot().unwrap().active_session,
        Some(first.clone())
    );
    assert_eq!(
        restored.events_after(0).unwrap().last().unwrap().sequence,
        first.last_sequence
    );

    let second = restored
        .run_session(
            &bridge,
            game_id(),
            Duration::from_secs(2),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(second.session_id.as_str(), "session-simulated-000002");
    assert!(second.last_sequence > first.last_sequence);
}

#[test]
fn sequence_gaps_and_unreconciled_sessions_fail_closed() {
    let gap_directory = TestDirectory::create();
    let gap_store = Arc::new(FileSessionEventStore::create(&gap_directory.0).unwrap());
    gap_store
        .append(&StoredSessionEvent::new(
            game_id(),
            SessionEvent {
                sequence: 2,
                session_id: SessionId::parse("session-simulated-000001").unwrap(),
                previous: SessionState::Requested,
                current: SessionState::Validating,
                outcome: None,
            },
        ))
        .unwrap();
    assert!(matches!(
        Core::with_event_store(gap_store),
        Err(CoreError::StoredSequenceGap)
    ));

    let active_directory = TestDirectory::create();
    let active_store = Arc::new(FileSessionEventStore::create(&active_directory.0).unwrap());
    active_store
        .append(&StoredSessionEvent::new(
            game_id(),
            SessionEvent {
                sequence: 1,
                session_id: SessionId::parse("session-simulated-000001").unwrap(),
                previous: SessionState::Requested,
                current: SessionState::Validating,
                outcome: None,
            },
        ))
        .unwrap();
    assert!(matches!(
        Core::with_event_store(active_store),
        Err(CoreError::UnreconciledStoredSession)
    ));
}
