#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use limen_domain::{GameId, SessionId, SessionOutcome, SessionState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionEvent {
    pub sequence: u64,
    pub session_id: SessionId,
    pub previous: SessionState,
    pub current: SessionState,
    pub outcome: Option<SessionOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMachine {
    session_id: SessionId,
    game_id: GameId,
    state: SessionState,
    outcome: Option<SessionOutcome>,
    sequence: u64,
}

impl SessionMachine {
    pub fn new(session_id: SessionId, game_id: GameId) -> Self {
        Self {
            session_id,
            game_id,
            state: SessionState::Requested,
            outcome: None,
            sequence: 0,
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn game_id(&self) -> &GameId {
        &self.game_id
    }

    pub const fn state(&self) -> SessionState {
        self.state
    }

    pub const fn outcome(&self) -> Option<SessionOutcome> {
        self.outcome
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn transition(&mut self, next: SessionState) -> Result<SessionEvent, TransitionError> {
        if !can_transition(self.state, next) {
            return Err(TransitionError {
                current: self.state,
                attempted: next,
            });
        }

        let previous = self.state;
        self.state = next;
        self.outcome = outcome_for(next).or(self.outcome);
        self.sequence = self.sequence.saturating_add(1);

        Ok(SessionEvent {
            sequence: self.sequence,
            session_id: self.session_id.clone(),
            previous,
            current: next,
            outcome: self.outcome,
        })
    }
}

pub const fn can_transition(current: SessionState, next: SessionState) -> bool {
    use SessionState as State;

    matches!(
        (current, next),
        (State::Requested, State::Validating)
            | (State::Requested, State::Cancelled)
            | (State::Validating, State::Preparing)
            | (State::Validating, State::Failed)
            | (State::Validating, State::Cancelled)
            | (State::Preparing, State::Launching)
            | (State::Preparing, State::Failed)
            | (State::Preparing, State::Cancelled)
            | (State::Launching, State::Running)
            | (State::Launching, State::Failed)
            | (State::Launching, State::TimedOut)
            | (State::Launching, State::Cancelled)
            | (State::Running, State::Stopping)
            | (State::Running, State::Crashed)
            | (State::Stopping, State::Finished)
            | (State::Stopping, State::Crashed)
            | (State::Stopping, State::TimedOut)
            | (State::Stopping, State::Cancelled)
            | (State::Finished, State::RecoveringHome)
            | (State::Crashed, State::RecoveringHome)
    )
}

const fn outcome_for(state: SessionState) -> Option<SessionOutcome> {
    match state {
        SessionState::Finished => Some(SessionOutcome::Finished),
        SessionState::Crashed => Some(SessionOutcome::Crashed),
        SessionState::Failed => Some(SessionOutcome::Failed),
        SessionState::Cancelled => Some(SessionOutcome::Cancelled),
        SessionState::TimedOut => Some(SessionOutcome::TimedOut),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransitionError {
    pub current: SessionState,
    pub attempted: SessionState,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot transition session from {} to {}",
            self.current.as_str(),
            self.attempted.as_str()
        )
    }
}

impl Error for TransitionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn machine() -> SessionMachine {
        SessionMachine::new(
            SessionId::parse("session-test-001").unwrap(),
            GameId::parse("game-placeholder").unwrap(),
        )
    }

    #[test]
    fn normal_lifecycle_has_monotonic_events_and_recovers_home() {
        let mut machine = machine();
        let states = [
            SessionState::Validating,
            SessionState::Preparing,
            SessionState::Launching,
            SessionState::Running,
            SessionState::Stopping,
            SessionState::Finished,
            SessionState::RecoveringHome,
        ];

        for (index, state) in states.into_iter().enumerate() {
            let event = machine.transition(state).unwrap();
            assert_eq!(event.sequence, (index + 1) as u64);
        }

        assert_eq!(machine.outcome(), Some(SessionOutcome::Finished));
        assert!(machine.state().is_terminal());
    }

    #[test]
    fn crash_and_timeout_are_distinct_results() {
        let mut crashed = machine();
        for state in [
            SessionState::Validating,
            SessionState::Preparing,
            SessionState::Launching,
            SessionState::Running,
            SessionState::Crashed,
            SessionState::RecoveringHome,
        ] {
            crashed.transition(state).unwrap();
        }

        let mut timed_out = machine();
        for state in [
            SessionState::Validating,
            SessionState::Preparing,
            SessionState::Launching,
            SessionState::Running,
            SessionState::Stopping,
            SessionState::TimedOut,
        ] {
            timed_out.transition(state).unwrap();
        }

        assert_eq!(crashed.outcome(), Some(SessionOutcome::Crashed));
        assert_eq!(timed_out.outcome(), Some(SessionOutcome::TimedOut));
    }

    #[test]
    fn invalid_transition_does_not_mutate_state_or_sequence() {
        let mut machine = machine();
        let error = machine.transition(SessionState::Running).unwrap_err();

        assert_eq!(error.current, SessionState::Requested);
        assert_eq!(machine.state(), SessionState::Requested);
        assert_eq!(machine.sequence(), 0);
    }
}
