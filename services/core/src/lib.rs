#![forbid(unsafe_code)]

mod api;
mod ipc_service;
mod supervisor;

use std::error::Error;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use limen_bridge_sdk::{Bridge, LaunchIntent};
use limen_contracts::{HomeSnapshot, SessionSnapshot};
use limen_domain::{GameId, InvalidIdentifier, SessionId, SessionState};
use limen_session::{SessionEvent, SessionMachine, TransitionError};

pub use api::{CoreApi, SimulatedSessionConfig};
pub use ipc_service::{CoreIpcService, CoreIpcServiceError};
pub use supervisor::{ProcessExit, ProcessSupervisor};

#[derive(Clone, Debug, Default)]
pub struct Core {
    state: Arc<Mutex<CoreState>>,
}

#[derive(Debug, Default)]
struct CoreState {
    active_session: Option<SessionMachine>,
    events: Vec<SessionEvent>,
    next_session_number: u64,
}

impl Core {
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
        let event = state
            .active_session
            .as_mut()
            .ok_or(CoreError::SessionNotFound)?
            .transition(next)?;
        state.events.push(event);
        Ok(())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, CoreState>, CoreError> {
        self.state.lock().map_err(|_| CoreError::StatePoisoned)
    }
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
    InvalidIdentifier(InvalidIdentifier),
    InvalidTransition(TransitionError),
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
            Self::InvalidIdentifier(_) => "Core generated an invalid internal identifier",
            Self::InvalidTransition(_) => "session state transition was rejected",
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

impl From<std::io::Error> for CoreError {
    fn from(error: std::io::Error) -> Self {
        Self::ProcessIo(error)
    }
}
