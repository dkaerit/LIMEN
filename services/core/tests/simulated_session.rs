use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use limen_bridge_fake::{FakeBridge, FakeRuntimeMode};
use limen_core::{Core, CoreError};
use limen_domain::{GameId, SessionOutcome, SessionState};

fn fake_runtime() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_limen-fake-runtime"))
}

fn game_id() -> GameId {
    GameId::parse("game-placeholder-001").unwrap()
}

#[test]
fn normal_exit_recovers_home_and_a_new_client_observes_the_same_snapshot() {
    let bridge = FakeBridge::new(fake_runtime(), FakeRuntimeMode::Normal);
    let mut core = Core::default();
    let result = core
        .run_session(
            &bridge,
            game_id(),
            Duration::from_secs(2),
            &AtomicBool::new(false),
        )
        .unwrap();

    let reconnected_home = core.home_snapshot();
    assert_eq!(result.state, SessionState::RecoveringHome);
    assert_eq!(result.outcome, Some(SessionOutcome::Finished));
    assert_eq!(reconnected_home.active_session, Some(result));
}

#[test]
fn crash_timeout_and_cancellation_have_distinct_outcomes() {
    let scenarios = [
        (
            FakeRuntimeMode::Crash,
            Duration::from_secs(2),
            false,
            SessionOutcome::Crashed,
        ),
        (
            FakeRuntimeMode::Hang,
            Duration::from_millis(40),
            false,
            SessionOutcome::TimedOut,
        ),
        (
            FakeRuntimeMode::Hang,
            Duration::from_secs(2),
            true,
            SessionOutcome::Cancelled,
        ),
    ];

    for (mode, timeout, cancel_later, expected) in scenarios {
        let bridge = FakeBridge::new(fake_runtime(), mode);
        let mut core = Core::default();
        let cancelled = Arc::new(AtomicBool::new(false));

        if cancel_later {
            let signal = Arc::clone(&cancelled);
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(30));
                signal.store(true, Ordering::Release);
            });
        }

        let result = core
            .run_session(&bridge, game_id(), timeout, cancelled.as_ref())
            .unwrap();
        assert_eq!(result.outcome, Some(expected));
    }
}

#[test]
fn a_second_session_is_rejected_while_the_first_is_active() {
    let mut core = Core::default();
    core.start_session(game_id()).unwrap();
    let error = core.start_session(game_id()).unwrap_err();

    assert!(matches!(error, CoreError::SessionAlreadyActive));
}

#[test]
fn event_replay_is_monotonic_and_can_resume_after_a_known_sequence() {
    let bridge = FakeBridge::new(fake_runtime(), FakeRuntimeMode::Normal);
    let mut core = Core::default();
    let result = core
        .run_session(
            &bridge,
            game_id(),
            Duration::from_secs(2),
            &AtomicBool::new(false),
        )
        .unwrap();
    let replay = core.events_after(3);

    assert!(!replay.is_empty());
    assert!(replay.iter().all(|event| event.sequence > 3));
    assert_eq!(replay.last().unwrap().sequence, result.last_sequence);
}
