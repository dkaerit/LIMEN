#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use limen_domain::{ClientId, GameId, RequestId, SessionId, SessionOutcome, SessionState};

pub const API_MAJOR: u16 = 1;
pub const MESSAGE_VERSION: u16 = 1;
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;
pub const EPHEMERAL_SECRET_BYTES: usize = 32;

#[derive(Clone, Eq, PartialEq)]
pub struct EphemeralSecret([u8; EPHEMERAL_SECRET_BYTES]);

impl EphemeralSecret {
    pub const fn new(bytes: [u8; EPHEMERAL_SECRET_BYTES]) -> Self {
        Self(bytes)
    }

    pub fn matches(&self, candidate: &[u8]) -> bool {
        if candidate.len() != EPHEMERAL_SECRET_BYTES {
            return false;
        }

        self.0
            .iter()
            .zip(candidate)
            .fold(0_u8, |difference, (left, right)| difference | (left ^ right))
            == 0
    }
}

impl fmt::Debug for EphemeralSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EphemeralSecret([REDACTED])")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Compatibility {
    pub api_major: u16,
    pub message_version: u16,
}

impl Compatibility {
    pub const CURRENT: Self = Self {
        api_major: API_MAJOR,
        message_version: MESSAGE_VERSION,
    };

    pub fn validate(self) -> Result<(), CompatibilityError> {
        if self.api_major != API_MAJOR {
            return Err(CompatibilityError::UnsupportedApiMajor {
                expected: API_MAJOR,
                received: self.api_major,
            });
        }
        if self.message_version == 0 || self.message_version > MESSAGE_VERSION {
            return Err(CompatibilityError::UnsupportedMessageVersion {
                maximum: MESSAGE_VERSION,
                received: self.message_version,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityError {
    UnsupportedApiMajor { expected: u16, received: u16 },
    UnsupportedMessageVersion { maximum: u16, received: u16 },
}

impl fmt::Display for CompatibilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedApiMajor { expected, received } => write!(
                formatter,
                "unsupported API major {received}; this Core requires {expected}"
            ),
            Self::UnsupportedMessageVersion { maximum, received } => write!(
                formatter,
                "unsupported message version {received}; maximum supported is {maximum}"
            ),
        }
    }
}

impl Error for CompatibilityError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandshakeRequest {
    pub compatibility: Compatibility,
    pub client_id: ClientId,
    pub client_name: String,
    pub capabilities: Vec<ClientCapability>,
    pub secret: EphemeralSecret,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientCapability {
    HomeSnapshot,
    SessionCommands,
    SessionEvents,
    DiagnosticsRead,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestEnvelope {
    pub compatibility: Compatibility,
    pub request_id: RequestId,
    pub payload: RequestPayload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestPayload {
    SystemGetInfo,
    LibraryGetHomeSnapshot,
    SessionStart { game_id: GameId },
    SessionGet { session_id: SessionId },
    SessionStop { session_id: SessionId },
    EventsSubscribe { after_sequence: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseEnvelope {
    pub compatibility: Compatibility,
    pub request_id: RequestId,
    pub payload: Result<ResponsePayload, ApiError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponsePayload {
    SystemInfo(SystemInfo),
    HomeSnapshot(HomeSnapshot),
    Session(SessionSnapshot),
    EventsSubscribed { current_sequence: u64 },
    Accepted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemInfo {
    pub core_version: String,
    pub api_major: u16,
    pub modules: Vec<ModuleStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleStatus {
    pub module: String,
    pub state: ModuleHealth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleHealth {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeSnapshot {
    pub selected_game_id: Option<GameId>,
    pub active_session: Option<SessionSnapshot>,
    pub last_event_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSnapshot {
    pub session_id: SessionId,
    pub game_id: GameId,
    pub state: SessionState,
    pub outcome: Option<SessionOutcome>,
    pub last_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventEnvelope {
    pub compatibility: Compatibility,
    pub sequence: u64,
    pub session_id: Option<SessionId>,
    pub payload: EventPayload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventPayload {
    SessionStateChanged {
        previous: SessionState,
        current: SessionState,
    },
    SessionOutcomeRecorded { outcome: SessionOutcome },
    ModuleHealthChanged { module: String, state: ModuleHealth },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub user_message: String,
    pub retryable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiErrorCode {
    AuthenticationFailed,
    IncompatibleClient,
    InvalidRequest,
    SessionAlreadyActive,
    SessionNotFound,
    InvalidSessionTransition,
    RuntimeCrashed,
    RuntimeTimedOut,
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incompatible_major_fails_closed() {
        let error = Compatibility {
            api_major: 2,
            message_version: 1,
        }
        .validate()
        .expect_err("v2 must not be accepted by the v1 Core");

        assert_eq!(
            error,
            CompatibilityError::UnsupportedApiMajor {
                expected: 1,
                received: 2
            }
        );
    }

    #[test]
    fn secret_debug_output_is_redacted_and_comparison_is_exact() {
        let secret = EphemeralSecret::new([7; EPHEMERAL_SECRET_BYTES]);

        assert!(secret.matches(&[7; EPHEMERAL_SECRET_BYTES]));
        assert!(!secret.matches(&[8; EPHEMERAL_SECRET_BYTES]));
        assert_eq!(format!("{secret:?}"), "EphemeralSecret([REDACTED])");
    }
}
