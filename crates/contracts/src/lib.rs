#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use limen_domain::{ClientId, GameId, RequestId, SessionId, SessionOutcome, SessionState};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
            .fold(0_u8, |difference, (left, right)| {
                difference | (left ^ right)
            })
            == 0
    }

    pub fn matches_secret(&self, candidate: &Self) -> bool {
        self.matches(&candidate.0)
    }
}

impl fmt::Debug for EphemeralSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EphemeralSecret([REDACTED])")
    }
}

impl Serialize for EphemeralSecret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&URL_SAFE_NO_PAD.encode(self.0))
    }
}

impl<'de> Deserialize<'de> for EphemeralSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        let decoded = URL_SAFE_NO_PAD
            .decode(encoded.as_bytes())
            .or_else(|_| URL_SAFE.decode(encoded.as_bytes()))
            .map_err(serde::de::Error::custom)?;
        let bytes = decoded.try_into().map_err(|decoded: Vec<u8>| {
            serde::de::Error::custom(format_args!(
                "ephemeral secret decoded to {} bytes; expected {EPHEMERAL_SECRET_BYTES}",
                decoded.len()
            ))
        })?;
        Ok(Self::new(bytes))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HandshakeRequest {
    pub api_major: u16,
    pub message_version: u16,
    pub client_id: ClientId,
    pub client_name: String,
    pub channel: ClientChannel,
    pub capabilities: Vec<ClientCapability>,
    pub secret: EphemeralSecret,
}

impl HandshakeRequest {
    pub const fn compatibility(&self) -> Compatibility {
        Compatibility {
            api_major: self.api_major,
            message_version: self.message_version,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientChannel {
    Commands,
    Events,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    HomeSnapshot,
    SessionCommands,
    SessionEvents,
    DiagnosticsRead,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope {
    pub api_major: u16,
    pub message_version: u16,
    pub request_id: RequestId,
    pub payload: RequestPayload,
}

impl RequestEnvelope {
    pub const fn compatibility(&self) -> Compatibility {
        Compatibility {
            api_major: self.api_major,
            message_version: self.message_version,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, tag = "method")]
pub enum RequestPayload {
    #[serde(rename = "system.get_info")]
    SystemGetInfo,
    #[serde(rename = "library.get_home_snapshot")]
    LibraryGetHomeSnapshot,
    #[serde(rename = "session.start")]
    SessionStart { game_id: GameId },
    #[serde(rename = "session.get")]
    SessionGet { session_id: SessionId },
    #[serde(rename = "session.stop")]
    SessionStop { session_id: SessionId },
    #[serde(rename = "events.subscribe")]
    EventsSubscribe { after_sequence: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEnvelope {
    pub api_major: u16,
    pub message_version: u16,
    pub request_id: RequestId,
    pub ok: bool,
    pub result: Option<ResponsePayload>,
    pub error: Option<ApiError>,
}

impl ResponseEnvelope {
    pub fn success(request_id: RequestId, result: ResponsePayload) -> Self {
        Self {
            api_major: API_MAJOR,
            message_version: MESSAGE_VERSION,
            request_id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(request_id: RequestId, error: ApiError) -> Self {
        Self {
            api_major: API_MAJOR,
            message_version: MESSAGE_VERSION,
            request_id,
            ok: false,
            result: None,
            error: Some(error),
        }
    }

    pub const fn is_valid(&self) -> bool {
        matches!(
            (self.ok, self.result.is_some(), self.error.is_some()),
            (true, true, false) | (false, false, true)
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(content = "data", tag = "type")]
pub enum ResponsePayload {
    #[serde(rename = "system.info")]
    SystemInfo(SystemInfo),
    #[serde(rename = "home.snapshot")]
    HomeSnapshot(HomeSnapshot),
    #[serde(rename = "session.snapshot")]
    Session(SessionSnapshot),
    #[serde(rename = "events.subscribed")]
    EventsSubscribed { current_sequence: u64 },
    #[serde(rename = "accepted")]
    Accepted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SystemInfo {
    pub core_version: String,
    pub api_major: u16,
    pub modules: Vec<ModuleStatus>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleStatus {
    pub module: String,
    pub state: ModuleHealth,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleHealth {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomeSnapshot {
    pub selected_game_id: Option<GameId>,
    pub active_session: Option<SessionSnapshot>,
    pub last_event_sequence: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSnapshot {
    pub session_id: SessionId,
    pub game_id: GameId,
    pub state: SessionState,
    pub outcome: Option<SessionOutcome>,
    pub last_sequence: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventEnvelope {
    pub api_major: u16,
    pub message_version: u16,
    pub sequence: u64,
    pub session_id: Option<SessionId>,
    #[serde(rename = "event")]
    pub payload: EventPayload,
}

impl EventEnvelope {
    pub const fn compatibility(&self) -> Compatibility {
        Compatibility {
            api_major: self.api_major,
            message_version: self.message_version,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum EventPayload {
    #[serde(rename = "session.state_changed")]
    SessionStateChanged {
        previous: SessionState,
        current: SessionState,
    },
    #[serde(rename = "session.outcome_recorded")]
    SessionOutcomeRecorded { outcome: SessionOutcome },
    #[serde(rename = "module.health_changed")]
    ModuleHealthChanged { module: String, state: ModuleHealth },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub user_message: String,
    pub retryable: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
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
