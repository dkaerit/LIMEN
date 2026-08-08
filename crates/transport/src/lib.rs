#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};

use limen_contracts::{
    ClientCapability, ClientChannel, CompatibilityError, EphemeralSecret, HandshakeRequest,
    MAX_FRAME_BYTES, RequestPayload,
};
use limen_domain::ClientId;

pub struct FrameCodec;

impl FrameCodec {
    pub fn read(reader: &mut impl Read) -> Result<Vec<u8>, FrameError> {
        let mut prefix = [0_u8; std::mem::size_of::<u32>()];
        reader.read_exact(&mut prefix)?;
        let length = u32::from_le_bytes(prefix) as usize;
        validate_length(length)?;

        let mut payload = vec![0_u8; length];
        reader.read_exact(&mut payload)?;
        Ok(payload)
    }

    pub fn write(writer: &mut impl Write, payload: &[u8]) -> Result<(), FrameError> {
        validate_length(payload.len())?;
        let length = u32::try_from(payload.len()).map_err(|_| FrameError::TooLarge {
            received: payload.len(),
            maximum: MAX_FRAME_BYTES,
        })?;

        writer.write_all(&length.to_le_bytes())?;
        writer.write_all(payload)?;
        Ok(())
    }
}

fn validate_length(length: usize) -> Result<(), FrameError> {
    if length == 0 {
        return Err(FrameError::Empty);
    }
    if length > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge {
            received: length,
            maximum: MAX_FRAME_BYTES,
        });
    }
    Ok(())
}

#[derive(Debug)]
pub enum FrameError {
    Empty,
    TooLarge { received: usize, maximum: usize },
    Io(io::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("local API frames cannot be empty"),
            Self::TooLarge { received, maximum } => {
                write!(
                    formatter,
                    "frame has {received} bytes; maximum is {maximum}"
                )
            }
            Self::Io(_) => formatter.write_str("local API frame could not be transferred"),
        }
    }
}

impl Error for FrameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for FrameError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandshakePolicy {
    expected_secret: EphemeralSecret,
    allowed_capabilities: Vec<ClientCapability>,
}

impl HandshakePolicy {
    pub fn new(
        expected_secret: EphemeralSecret,
        allowed_capabilities: Vec<ClientCapability>,
    ) -> Self {
        Self {
            expected_secret,
            allowed_capabilities,
        }
    }

    pub fn authenticate(
        &self,
        request: HandshakeRequest,
    ) -> Result<AuthenticatedClient, HandshakeError> {
        if !self.expected_secret.matches_secret(&request.secret) {
            return Err(HandshakeError::AuthenticationFailed);
        }
        request
            .compatibility
            .validate()
            .map_err(HandshakeError::IncompatibleClient)?;

        let client_name_length = request.client_name.chars().count();
        if !(1..=80).contains(&client_name_length) {
            return Err(HandshakeError::InvalidClientName);
        }

        for (index, capability) in request.capabilities.iter().enumerate() {
            if request.capabilities[..index].contains(capability) {
                return Err(HandshakeError::DuplicateCapability(*capability));
            }
            if !self.allowed_capabilities.contains(capability) {
                return Err(HandshakeError::CapabilityDenied(*capability));
            }
        }

        Ok(AuthenticatedClient {
            client_id: request.client_id,
            client_name: request.client_name,
            channel: request.channel,
            capabilities: request.capabilities,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedClient {
    pub client_id: ClientId,
    pub client_name: String,
    pub channel: ClientChannel,
    capabilities: Vec<ClientCapability>,
}

impl AuthenticatedClient {
    pub fn capabilities(&self) -> &[ClientCapability] {
        &self.capabilities
    }

    pub fn authorize(&self, request: &RequestPayload) -> Result<(), AuthorizationError> {
        match (self.channel, request) {
            (ClientChannel::Events, RequestPayload::EventsSubscribe { .. }) => self.require_any(&[
                ClientCapability::SessionEvents,
                ClientCapability::DiagnosticsRead,
            ]),
            (ClientChannel::Events, _)
            | (ClientChannel::Commands, RequestPayload::EventsSubscribe { .. }) => {
                Err(AuthorizationError::WrongChannel)
            }
            (ClientChannel::Commands, RequestPayload::SystemGetInfo) => Ok(()),
            (ClientChannel::Commands, RequestPayload::LibraryGetHomeSnapshot) => {
                self.require(ClientCapability::HomeSnapshot)
            }
            (ClientChannel::Commands, RequestPayload::SessionStart { .. })
            | (ClientChannel::Commands, RequestPayload::SessionStop { .. }) => {
                self.require(ClientCapability::SessionCommands)
            }
            (ClientChannel::Commands, RequestPayload::SessionGet { .. }) => self.require_any(&[
                ClientCapability::HomeSnapshot,
                ClientCapability::SessionCommands,
                ClientCapability::DiagnosticsRead,
            ]),
        }
    }

    fn require(&self, required: ClientCapability) -> Result<(), AuthorizationError> {
        if self.capabilities.contains(&required) {
            Ok(())
        } else {
            Err(AuthorizationError::MissingCapability(required))
        }
    }

    fn require_any(&self, accepted: &[ClientCapability]) -> Result<(), AuthorizationError> {
        if accepted
            .iter()
            .any(|capability| self.capabilities.contains(capability))
        {
            Ok(())
        } else {
            Err(AuthorizationError::MissingAnyCapability)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandshakeError {
    AuthenticationFailed,
    IncompatibleClient(CompatibilityError),
    InvalidClientName,
    DuplicateCapability(ClientCapability),
    CapabilityDenied(ClientCapability),
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationFailed => formatter.write_str("client authentication failed"),
            Self::IncompatibleClient(error) => error.fmt(formatter),
            Self::InvalidClientName => {
                formatter.write_str("client name must contain 1-80 characters")
            }
            Self::DuplicateCapability(_) => formatter.write_str("client repeated a capability"),
            Self::CapabilityDenied(_) => {
                formatter.write_str("client requested a capability that is not allowed")
            }
        }
    }
}

impl Error for HandshakeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IncompatibleClient(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
    WrongChannel,
    MissingCapability(ClientCapability),
    MissingAnyCapability,
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongChannel => {
                formatter.write_str("request is not valid on this connection channel")
            }
            Self::MissingCapability(_) | Self::MissingAnyCapability => {
                formatter.write_str("client is not authorized for this request")
            }
        }
    }
}

impl Error for AuthorizationError {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use limen_contracts::{Compatibility, EPHEMERAL_SECRET_BYTES};
    use limen_domain::{GameId, SessionId};

    use super::*;

    fn handshake(
        secret: EphemeralSecret,
        channel: ClientChannel,
        capabilities: Vec<ClientCapability>,
    ) -> HandshakeRequest {
        HandshakeRequest {
            compatibility: Compatibility::CURRENT,
            client_id: ClientId::parse("client-test-001").unwrap(),
            client_name: "LIMEN test client".to_owned(),
            channel,
            capabilities,
            secret,
        }
    }

    #[test]
    fn frame_round_trip_uses_little_endian_length() {
        let payload = br#"{"api_major":1}"#;
        let mut wire = Vec::new();
        FrameCodec::write(&mut wire, payload).unwrap();

        assert_eq!(&wire[..4], &(payload.len() as u32).to_le_bytes());
        assert_eq!(FrameCodec::read(&mut Cursor::new(wire)).unwrap(), payload);
    }

    #[test]
    fn oversized_length_is_rejected_before_allocating_or_reading_a_body() {
        let length = (MAX_FRAME_BYTES as u32 + 1).to_le_bytes();
        let error = FrameCodec::read(&mut Cursor::new(length)).unwrap_err();

        assert!(matches!(error, FrameError::TooLarge { .. }));
    }

    #[test]
    fn empty_and_truncated_frames_fail_closed() {
        assert!(matches!(
            FrameCodec::read(&mut Cursor::new(0_u32.to_le_bytes())).unwrap_err(),
            FrameError::Empty
        ));

        let mut truncated = 4_u32.to_le_bytes().to_vec();
        truncated.extend_from_slice(b"{}");
        let error = FrameCodec::read(&mut Cursor::new(truncated)).unwrap_err();
        assert!(matches!(error, FrameError::Io(_)));
    }

    #[test]
    fn invalid_secret_and_ungranted_capability_fail_authentication() {
        let expected = EphemeralSecret::new([7; EPHEMERAL_SECRET_BYTES]);
        let policy = HandshakePolicy::new(expected, vec![ClientCapability::DiagnosticsRead]);

        let wrong_secret = policy
            .authenticate(handshake(
                EphemeralSecret::new([8; EPHEMERAL_SECRET_BYTES]),
                ClientChannel::Commands,
                vec![ClientCapability::DiagnosticsRead],
            ))
            .unwrap_err();
        assert_eq!(wrong_secret, HandshakeError::AuthenticationFailed);

        let denied = policy
            .authenticate(handshake(
                EphemeralSecret::new([7; EPHEMERAL_SECRET_BYTES]),
                ClientChannel::Commands,
                vec![ClientCapability::SessionCommands],
            ))
            .unwrap_err();
        assert_eq!(
            denied,
            HandshakeError::CapabilityDenied(ClientCapability::SessionCommands)
        );
    }

    #[test]
    fn diagnostics_client_can_observe_but_cannot_start_or_stop_sessions() {
        let secret = EphemeralSecret::new([7; EPHEMERAL_SECRET_BYTES]);
        let policy = HandshakePolicy::new(secret.clone(), vec![ClientCapability::DiagnosticsRead]);
        let commands = policy
            .authenticate(handshake(
                secret,
                ClientChannel::Commands,
                vec![ClientCapability::DiagnosticsRead],
            ))
            .unwrap();

        assert!(commands.authorize(&RequestPayload::SystemGetInfo).is_ok());
        assert!(
            commands
                .authorize(&RequestPayload::SessionGet {
                    session_id: SessionId::parse("session-test-001").unwrap(),
                })
                .is_ok()
        );
        assert_eq!(
            commands
                .authorize(&RequestPayload::SessionStart {
                    game_id: GameId::parse("game-test-001").unwrap(),
                })
                .unwrap_err(),
            AuthorizationError::MissingCapability(ClientCapability::SessionCommands)
        );
    }

    #[test]
    fn events_require_the_dedicated_channel() {
        let secret = EphemeralSecret::new([7; EPHEMERAL_SECRET_BYTES]);
        let policy = HandshakePolicy::new(secret.clone(), vec![ClientCapability::SessionEvents]);
        let command_client = policy
            .authenticate(handshake(
                secret.clone(),
                ClientChannel::Commands,
                vec![ClientCapability::SessionEvents],
            ))
            .unwrap();
        let event_client = policy
            .authenticate(handshake(
                secret,
                ClientChannel::Events,
                vec![ClientCapability::SessionEvents],
            ))
            .unwrap();
        let subscribe = RequestPayload::EventsSubscribe { after_sequence: 0 };

        assert_eq!(
            command_client.authorize(&subscribe).unwrap_err(),
            AuthorizationError::WrongChannel
        );
        assert!(event_client.authorize(&subscribe).is_ok());
    }
}
