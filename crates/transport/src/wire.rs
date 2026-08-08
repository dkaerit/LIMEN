use std::error::Error;
use std::fmt;
use std::io::{Read, Write};

use limen_contracts::{EPHEMERAL_SECRET_BYTES, EphemeralSecret};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{FrameCodec, FrameError};

pub struct JsonFrameCodec;

impl JsonFrameCodec {
    pub fn read<T>(reader: &mut impl Read) -> Result<T, JsonFrameError>
    where
        T: DeserializeOwned,
    {
        let payload = FrameCodec::read(reader)?;
        serde_json::from_slice(&payload).map_err(JsonFrameError::InvalidJson)
    }

    pub fn write<T>(writer: &mut impl Write, message: &T) -> Result<(), JsonFrameError>
    where
        T: Serialize,
    {
        let payload = serde_json::to_vec(message).map_err(JsonFrameError::InvalidJson)?;
        FrameCodec::write(writer, &payload).map_err(JsonFrameError::Frame)
    }
}

#[derive(Debug)]
pub enum JsonFrameError {
    Frame(FrameError),
    InvalidJson(serde_json::Error),
}

impl fmt::Display for JsonFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Frame(error) => error.fmt(formatter),
            Self::InvalidJson(_) => formatter.write_str("local API message is not valid JSON"),
        }
    }
}

impl Error for JsonFrameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frame(error) => Some(error),
            Self::InvalidJson(error) => Some(error),
        }
    }
}

impl From<FrameError> for JsonFrameError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

pub fn generate_ephemeral_secret() -> Result<EphemeralSecret, SecretGenerationError> {
    let mut bytes = [0_u8; EPHEMERAL_SECRET_BYTES];
    getrandom::fill(&mut bytes).map_err(SecretGenerationError)?;
    Ok(EphemeralSecret::new(bytes))
}

#[derive(Debug)]
pub struct SecretGenerationError(getrandom::Error);

impl fmt::Display for SecretGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("operating system could not generate an ephemeral secret")
    }
}

impl Error for SecretGenerationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use limen_contracts::{
        API_MAJOR, MESSAGE_VERSION, RequestEnvelope, RequestPayload,
    };
    use limen_domain::{GameId, RequestId};

    use super::*;

    #[test]
    fn request_round_trip_keeps_the_neutral_schema_shape() {
        let request = RequestEnvelope {
            api_major: API_MAJOR,
            message_version: MESSAGE_VERSION,
            request_id: RequestId::parse("request-test-001").unwrap(),
            payload: RequestPayload::SessionStart {
                game_id: GameId::parse("game-test-001").unwrap(),
            },
        };
        let mut wire = Vec::new();
        JsonFrameCodec::write(&mut wire, &request).unwrap();
        let decoded: RequestEnvelope = JsonFrameCodec::read(&mut Cursor::new(wire)).unwrap();

        assert_eq!(decoded, request);
    }

    #[test]
    fn invalid_domain_identifier_and_unknown_field_fail_closed() {
        let invalid_identifier = br#"{
            "api_major": 1,
            "message_version": 1,
            "request_id": "../escape",
            "payload": {"method": "system.get_info"}
        }"#;
        let unknown_field = br#"{
            "api_major": 1,
            "message_version": 1,
            "request_id": "request-test-001",
            "unexpected": true,
            "payload": {"method": "system.get_info"}
        }"#;

        for payload in [invalid_identifier.as_slice(), unknown_field.as_slice()] {
            let mut wire = Vec::new();
            FrameCodec::write(&mut wire, payload).unwrap();
            let error = JsonFrameCodec::read::<RequestEnvelope>(&mut Cursor::new(wire)).unwrap_err();
            assert!(matches!(error, JsonFrameError::InvalidJson(_)));
        }
    }

    #[test]
    fn generated_secrets_are_serializable_and_debug_redacted() {
        let secret = generate_ephemeral_secret().unwrap();
        let encoded = serde_json::to_string(&secret).unwrap();
        let decoded: EphemeralSecret = serde_json::from_str(&encoded).unwrap();

        assert!(secret.matches_secret(&decoded));
        assert!(!encoded.contains("REDACTED"));
        assert_eq!(format!("{secret:?}"), "EphemeralSecret([REDACTED])");
    }
}
