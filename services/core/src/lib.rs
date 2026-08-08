#![forbid(unsafe_code)]

mod api;
mod ipc_service;
mod persistence;
mod supervisor;

use std::error::Error;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use limen_bridge_sdk::{Bridge, LaunchIntent};
use limen_contracts::{HomeSnapshot, SessionSnapshot};
use limen_domain::{GameId, InvalidIdentifier, SessionId, SessionState};
use limen_session::{RestoreError, SessionEvent, SessionMachine, TransitionError};

pub use api::{CoreApi, SimulatedSessionConfig};
pub use ipc_service::{CoreIpcService, CoreIpcServiceError};
pub use persistence::{
    FileSessionEventStore, PersistenceError, SessionEventStore, StoredSessionEvent,
};
pub use supervisor::{ProcessExit, ProcessSupervisor};

#[derive(Clone)]
pub struct Core {
    state: Arc<Mutex<CoreState>>,
    event_store: Option<Arc<dyn SessionEventStore>>,
}

#[derive(Debug, Default)]
struct CoreState {
    active_session: Option<SessionMachine>,
    events: Vec<SessionEvent>,
    next_session_number: u64,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(CoreState::default())),
            event_store: None,
        }
    }
}

impl fmt::Debug for Core {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Core")
            .field("state", &self.state)
            .field("persistent", &self.event_store.is_some())
            .finish()
    }
}

impl Core {
    pub fn with_event_store(store: Arc<dyn SessionEventStore>) -> Result<Self, CoreError> {
        let records = store.load()?;
        let state = restore_state(records)?;
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            event_store: Some(store),
        })
    }

    pub fn start_session(&self, game_id: GameId) -> Result<SessionId, CoreError> {
        let mut state = self.lock_state()?;
        if state
            .active_session
            .as_ref()
            .is_some_and(|session| !session.state().is_terminal())
        {
            return Err(CoreError::SessionAlreadyActive);
        }

        state.next_session_number = state.next_session_number.saturating_add(1);
        let session_id = SessionId::parse(format!(
            "session-simulated-{:06}",
            state.next_session_number
        ))?;
        let sequence = state.events.last().map_or(0, |event| event.sequence);
        state.active_session = Some(SessionMachine::new_at(
            session_id.clone(),
            game_id,
            sequence,
        ));
        Ok(session_id)
    }

    pub fn run_session<B: Bridge>(
        &self,
        bridge: &B,
        game_id: GameId,
        timeout: Duration,
        cancelled: &AtomicBool,
    ) -> Result<SessionSnapshot, CoreError> {
        let session_id = self.start_session(game_id.clone())?;
        self.continue_session(bridge, &session_id, game_id, timeout, cancelled)
    }

    pub fn continue_session<B: Bridge>(
        &self,
        bridge: &B,
        session_id: &SessionId,
        game_id: GameId,
        timeout: Duration,
        cancelled: &AtomicBool,
    ) -> Result<SessionSnapshot, CoreError> {
        self.ensure_active_session(session_id)?;
        self.transition(SessionState::Validating)?;

        let intent = LaunchIntent { game_id };
        if bridge.validate(&intent).is_err() {
            self.transition(SessionState::Failed)?;
            return Err(CoreError::BridgeValidationFailed);
        }

        self.transition(SessionState::Preparing)?;
        let plan = match bridge.plan_launch(&intent) {
            Ok(plan) => plan,
            Err(_) => {
                self.transition(SessionState::Failed)?;
                return Err(CoreError::BridgePlanFailed);
            }
        };

        self.transition(SessionState::Launching)?;
        let mut process = match ProcessSupervisor.start(&plan) {
            Ok(process) => process,
            Err(_) => {
                self.transition(SessionState::Failed)?;
                return Err(CoreError::ProcessStartFailed);
            }
        };
        self.transition(SessionState::Running)?;

        match process.wait(timeout, cancelled)? {
            ProcessExit::Normal => {
                self.transition(SessionState::Stopping)?;
                self.transition(SessionState::Finished)?;
                self.transition(SessionState::RecoveringHome)?;
            }
            ProcessExit::Crashed { .. } => {
                self.transition(SessionState::Crashed)?;
                self.transition(SessionState::RecoveringHome)?;
            }
            ProcessExit::TimedOut => {
                self.transition(SessionState::Stopping)?;
                self.transition(SessionState::TimedOut)?;
            }
            ProcessExit::Cancelled => {
                self.transition(SessionState::Stopping)?;
                self.transition(SessionState::Cancelled)?;
            }
        }

        self.session_snapshot_for(session_id)?
            .ok_or(CoreError::SessionNotFound)
    }

    pub fn home_snapshot(&self) -> Result<HomeSnapshot, CoreError> {
        let state = self.lock_state()?;
        let active_session = state.active_session.as_ref().map(session_snapshot);
        Ok(HomeSnapshot {
            selected_game_id: active_session
                .as_ref()
                .map(|session| session.game_id.clone()),
            active_session,
            last_event_sequence: state.events.last().map_or(0, |event| event.sequence),
        })
    }

    pub fn session_snapshot_for(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<SessionSnapshot>, CoreError> {
        let state = self.lock_state()?;
        Ok(state
            .active_session
            .as_ref()
            .filter(|session| session.session_id() == session_id)
            .map(session_snapshot))
    }

    pub fn events_after(&self, sequence: u64) -> Result<Vec<SessionEvent>, CoreError> {
        let state = self.lock_state()?;
        Ok(state
            .events
            .iter()
            .filter(|event| event.sequence > sequence)
            .cloned()
            .collect())
    }

    pub fn last_event_sequence(&self) -> Result<u64, CoreError> {
        let state = self.lock_state()?;
        Ok(state.events.last().map_or(0, |event| event.sequence))
    }

    fn ensure_active_session(&self, session_id: &SessionId) -> Result<(), CoreError> {
        let state = self.lock_state()?;
        let matches = state
            .active_session
            .as_ref()
            .is_some_and(|session| session.session_id() == session_id);
        if matches {
            Ok(())
        } else {
            Err(CoreError::SessionNotFound)
        }
    }

    fn transition(&self, next: SessionState) -> Result<(), CoreError> {
        let mut state = self.lock_state()?;
        let mut candidate = state
            .active_session
            .as_ref()
            .cloned()
            .ok_or(CoreError::SessionNotFound)?;
        let event = candidate.transition(next)?;
        if let Some(store) = &self.event_store {
            store.append(&StoredSessionEvent::new(
                candidate.game_id().clone(),
                event.clone(),
            ))?;
        }
        state.active_session = Some(candidate);
        state.events.push(event);
        Ok(())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, CoreState>, CoreError> {
        self.state.lock().map_err(|_| CoreError::StatePoisoned)
    }
}

fn restore_state(records: Vec<StoredSessionEvent>) -> Result<CoreState, CoreError> {
    let mut state = CoreState::default();

    for record in records {
        let expected_sequence = state.events.last().map_or(1, |event| {
            event.sequence.saturating_add(1)
        });
        if record.event.sequence != expected_sequence {
            return Err(CoreError::StoredSequenceGap);
        }

        let starts_new_session = state.active_session.as_ref().is_none_or(|session| {
            session.session_id() != &record.event.session_id
        });
        if starts_new_session {
            if state
                .active_session
                .as_ref()
                .is_some_and(|session| !session.state().is_terminal())
            {
                return Err(CoreError::StoredSessionOverlap);
            }
            let initial_sequence = record
                .event
                .sequence
                .checked_sub(1)
                .ok_or(CoreError::StoredSequenceGap)?;
            state.next_session_number = state.next_session_number.saturating_add(1);
            state.active_session = Some(SessionMachine::new_at(
                record.event.session_id.clone(),
                record.game_id.clone(),
                initial_sequence,
            ));
        }

        let session = state
            .active_session
            .as_mut()
            .ok_or(CoreError::SessionNotFound)?;
        if session.game_id() != &record.game_id {
            return Err(CoreError::StoredGameMismatch);
        }
        session.restore_event(&record.event)?;
        state.events.push(record.event);
    }

    if state
        .active_session
        .as_ref()
        .is_some_and(|session| !session.state().is_terminal())
    {
        return Err(CoreError::UnreconciledStoredSession);
    }
    Ok(state)
}

fn session_snapshot(session: &SessionMachine) -> SessionSnapshot {
    SessionSnapshot {
        session_id: session.session_id().clone(),
        game_id: session.game_id().clone(),
        state: session.state(),
        outcome: session.outcome(),
        last_sequence: session.sequence(),
    }
}

#[derive(Debug)]
pub enum CoreError {
    SessionAlreadyActive,
    SessionNotFound,
    BridgeValidationFailed,
    BridgePlanFailed,
    ProcessStartFailed,
    StatePoisoned,
    StoredSequenceGap,
    StoredSessionOverlap,
    StoredGameMismatch,
    UnreconciledStoredSession,
    InvalidIdentifier(InvalidIdentifier),
    InvalidTransition(TransitionError),
    InvalidStoredSession(RestoreError),
    Persistence(PersistenceError),
    ProcessIo(std::io::Error),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SessionAlreadyActive => "a session is already active",
            Self::SessionNotFound => "session was not found",
            Self::BridgeValidationFailed => "the Bridge rejected the launch intent",
            Self::BridgePlanFailed => "the Bridge could not create a launch plan",
            Self::ProcessStartFailed => "the supervised process could not be started",
            Self::StatePoisoned => "Core state could not be accessed safely",
            Self::StoredSequenceGap => "stored session events contain a sequence gap",
            Self::StoredSessionOverlap => "stored sessions overlap",
            Self::StoredGameMismatch => "stored session changed its game identifier",
            Self::UnreconciledStoredSession => {
                "stored session requires process reconciliation before Core can start"
            }
            Self::InvalidIdentifier(_) => "Core generated an invalid internal identifier",
            Self::InvalidTransition(_) => "session state transition was rejected",
            Self::InvalidStoredSession(_) => "stored session events are inconsistent",
            Self::Persistence(_) => "Core persistence failed",
            Self::ProcessIo(_) => "the supervised process failed",
        };
        formatter.write_str(message)
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidIdentifier(error) => Some(error),
            Self::InvalidTransition(error) => Some(error),
            Self::InvalidStoredSession(error) => Some(error),
            Self::Persistence(error) => Some(error),
            Self::ProcessIo(error) => Some(error),
            _ => None,
        }
    }
}

impl From<InvalidIdentifier> for CoreError {
    fn from(error: InvalidIdentifier) -> Self {
        Self::InvalidIdentifier(error)
    }
}

impl From<TransitionError> for CoreError {
    fn from(error: TransitionError) -> Self {
        Self::InvalidTransition(error)
    }
}

impl From<RestoreError> for CoreError {
    fn from(error: RestoreError) -> Self {
        Self::InvalidStoredSession(error)
    }
}

impl From<PersistenceError> for CoreError {
    fn from(error: PersistenceError) -> Self {
        Self::Persistence(error)
    }
}

impl From<std::io::Error> for CoreError {
    fn from(error: std::io::Error) -> Self {
        Self::ProcessIo(error)
    }
}
