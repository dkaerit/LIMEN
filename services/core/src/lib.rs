#![forbid(unsafe_code)]

mod supervisor;

use std::error::Error;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use limen_bridge_sdk::{Bridge, LaunchIntent};
use limen_contracts::{HomeSnapshot, SessionSnapshot};
use limen_domain::{GameId, InvalidIdentifier, SessionId, SessionState};
use limen_session::{SessionEvent, SessionMachine, TransitionError};
pub use supervisor::{ProcessExit, ProcessSupervisor};

#[derive(Debug, Default)]
pub struct Core {
    active_session: Option<SessionMachine>,
    events: Vec<SessionEvent>,
    next_session_number: u64,
}

impl Core {
    pub fn start_session(&mut self, game_id: GameId) -> Result<SessionId, CoreError> {
        if self
            .active_session
            .as_ref()
            .is_some_and(|session| !session.state().is_terminal())
        {
            return Err(CoreError::SessionAlreadyActive);
        }

        self.next_session_number = self.next_session_number.saturating_add(1);
        let session_id = SessionId::parse(format!(
            "session-simulated-{:06}",
            self.next_session_number
        ))?;
        self.active_session = Some(SessionMachine::new(session_id.clone(), game_id));
        Ok(session_id)
    }

    pub fn run_session<B: Bridge>(
        &mut self,
        bridge: &B,
        game_id: GameId,
        timeout: Duration,
        cancelled: &AtomicBool,
    ) -> Result<SessionSnapshot, CoreError> {
        self.start_session(game_id.clone())?;
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

        self.session_snapshot().ok_or(CoreError::SessionNotFound)
    }

    pub fn home_snapshot(&self) -> HomeSnapshot {
        let active_session = self.session_snapshot();
        HomeSnapshot {
            selected_game_id: active_session
                .as_ref()
                .map(|session| session.game_id.clone()),
            last_event_sequence: active_session
                .as_ref()
                .map_or(0, |session| session.last_sequence),
            active_session,
        }
    }

    pub fn events_after(&self, sequence: u64) -> Vec<SessionEvent> {
        self.events
            .iter()
            .filter(|event| event.sequence > sequence)
            .cloned()
            .collect()
    }

    fn transition(&mut self, next: SessionState) -> Result<(), CoreError> {
        let session = self
            .active_session
            .as_mut()
            .ok_or(CoreError::SessionNotFound)?;
        let event = session.transition(next)?;
        self.events.push(event);
        Ok(())
    }

    fn session_snapshot(&self) -> Option<SessionSnapshot> {
        self.active_session.as_ref().map(|session| SessionSnapshot {
            session_id: session.session_id().clone(),
            game_id: session.game_id().clone(),
            state: session.state(),
            outcome: session.outcome(),
            last_sequence: session.sequence(),
        })
    }
}

#[derive(Debug)]
pub enum CoreError {
    SessionAlreadyActive,
    SessionNotFound,
    BridgeValidationFailed,
    BridgePlanFailed,
    ProcessStartFailed,
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
