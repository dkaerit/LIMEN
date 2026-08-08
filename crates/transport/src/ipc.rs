use std::error::Error;
use std::fmt;
use std::io::{self, Write};

use interprocess::local_socket::{
    GenericNamespaced, Listener, ListenerOptions, Stream, prelude::*,
};
use limen_contracts::{CompatibilityError, HandshakeRequest, RequestEnvelope, ResponseEnvelope};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{
    AuthenticatedClient, AuthorizationError, HandshakeError, HandshakePolicy, JsonFrameCodec,
    JsonFrameError,
};

const MAX_ENDPOINT_BYTES: usize = 96;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointName(String);

impl EndpointName {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidEndpointName> {
        let value = value.into();
        let valid_length = !value.is_empty() && value.len() <= MAX_ENDPOINT_BYTES;
        let valid_characters = value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));

        if !valid_length || !valid_characters {
            return Err(InvalidEndpointName);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidEndpointName;

impl fmt::Display for InvalidEndpointName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("endpoint name must contain 1-96 portable characters")
    }
}

impl Error for InvalidEndpointName {}

pub struct IpcServer {
    listener: Listener,
}

impl IpcServer {
    pub fn bind(endpoint: &EndpointName) -> Result<Self, IpcError> {
        if !GenericNamespaced::is_supported() {
            return Err(IpcError::UnsupportedPlatform);
        }
        let name = endpoint.as_str().to_ns_name::<GenericNamespaced>()?;
        let listener = ListenerOptions::new().name(name).create_sync()?;
        Ok(Self { listener })
    }

    pub fn accept(&self) -> Result<IpcConnection, IpcError> {
        self.listener
            .accept()
            .map(IpcConnection::new)
            .map_err(IpcError::Io)
    }

    pub fn accept_authenticated(
        &self,
        policy: &HandshakePolicy,
    ) -> Result<AuthenticatedIpcConnection, IpcError> {
        let mut connection = self.accept()?;
        let handshake = connection.receive::<HandshakeRequest>()?;
        let client = policy.authenticate(handshake)?;
        Ok(AuthenticatedIpcConnection { connection, client })
    }
}

pub struct IpcConnection {
    stream: Stream,
}

impl IpcConnection {
    fn new(stream: Stream) -> Self {
        Self { stream }
    }

    pub fn connect(endpoint: &EndpointName) -> Result<Self, IpcError> {
        if !GenericNamespaced::is_supported() {
            return Err(IpcError::UnsupportedPlatform);
        }
        let name = endpoint.as_str().to_ns_name::<GenericNamespaced>()?;
        Stream::connect(name).map(Self::new).map_err(IpcError::Io)
    }

    pub fn receive<T>(&mut self) -> Result<T, IpcError>
    where
        T: DeserializeOwned,
    {
        JsonFrameCodec::read(&mut self.stream).map_err(IpcError::Wire)
    }

    pub fn send<T>(&mut self, message: &T) -> Result<(), IpcError>
    where
        T: Serialize,
    {
        JsonFrameCodec::write(&mut self.stream, message)?;
        self.stream.flush()?;
        Ok(())
    }
}

pub struct AuthenticatedIpcConnection {
    connection: IpcConnection,
    client: AuthenticatedClient,
}

impl AuthenticatedIpcConnection {
    pub fn client(&self) -> &AuthenticatedClient {
        &self.client
    }

    pub fn receive_request(&mut self) -> Result<RequestEnvelope, IpcError> {
        let request = self.connection.receive::<RequestEnvelope>()?;
        request.compatibility().validate()?;
        self.client.authorize(&request.payload)?;
        Ok(request)
    }

    pub fn send_response(&mut self, response: &ResponseEnvelope) -> Result<(), IpcError> {
        if !response.is_valid() {
            return Err(IpcError::InvalidResponseInvariant);
        }
        self.connection.send(response)
    }
}

#[derive(Debug)]
pub enum IpcError {
    UnsupportedPlatform,
    Io(io::Error),
    Wire(JsonFrameError),
    Handshake(HandshakeError),
    IncompatibleClient(CompatibilityError),
    Unauthorized(AuthorizationError),
    InvalidResponseInvariant,
}

impl fmt::Display for IpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("local namespaced sockets are unavailable on this platform")
            }
            Self::Io(_) => formatter.write_str("local IPC transport failed"),
            Self::Wire(error) => error.fmt(formatter),
            Self::Handshake(error) => error.fmt(formatter),
            Self::IncompatibleClient(error) => error.fmt(formatter),
            Self::Unauthorized(error) => error.fmt(formatter),
            Self::InvalidResponseInvariant => {
                formatter.write_str("Core attempted to send an invalid response envelope")
            }
        }
    }
}

impl Error for IpcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Wire(error) => Some(error),
            Self::Handshake(error) => Some(error),
            Self::IncompatibleClient(error) => Some(error),
            Self::Unauthorized(error) => Some(error),
            Self::UnsupportedPlatform | Self::InvalidResponseInvariant => None,
        }
    }
}

impl From<io::Error> for IpcError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<JsonFrameError> for IpcError {
    fn from(error: JsonFrameError) -> Self {
        Self::Wire(error)
    }
}

impl From<HandshakeError> for IpcError {
    fn from(error: HandshakeError) -> Self {
        Self::Handshake(error)
    }
}

impl From<CompatibilityError> for IpcError {
    fn from(error: CompatibilityError) -> Self {
        Self::IncompatibleClient(error)
    }
}

impl From<AuthorizationError> for IpcError {
    fn from(error: AuthorizationError) -> Self {
        Self::Unauthorized(error)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;

    use limen_contracts::{
        API_MAJOR, ClientCapability, ClientChannel, Compatibility, EPHEMERAL_SECRET_BYTES,
        EphemeralSecret, HandshakeRequest, MESSAGE_VERSION, RequestPayload, ResponsePayload,
    };
    use limen_domain::{ClientId, GameId, RequestId};

    use super::*;

    static NEXT_ENDPOINT: AtomicU64 = AtomicU64::new(1);

    fn endpoint() -> EndpointName {
        let sequence = NEXT_ENDPOINT.fetch_add(1, Ordering::Relaxed);
        EndpointName::parse(format!("limen-test-{}-{sequence}", std::process::id())).unwrap()
    }

    fn handshake(secret: EphemeralSecret, capabilities: Vec<ClientCapability>) -> HandshakeRequest {
        HandshakeRequest {
            api_major: Compatibility::CURRENT.api_major,
            message_version: Compatibility::CURRENT.message_version,
            client_id: ClientId::parse("client-ipc-test").unwrap(),
            client_name: "IPC test client".to_owned(),
            channel: ClientChannel::Commands,
            capabilities,
            secret,
        }
    }

    #[test]
    fn authenticated_socket_round_trip_uses_typed_json_frames() {
        let endpoint = endpoint();
        let server = IpcServer::bind(&endpoint).unwrap();
        let secret = EphemeralSecret::new([7; EPHEMERAL_SECRET_BYTES]);
        let client_endpoint = endpoint.clone();
        let client_secret = secret.clone();

        let client = thread::spawn(move || {
            let mut connection = IpcConnection::connect(&client_endpoint).unwrap();
            connection
                .send(&handshake(
                    client_secret,
                    vec![ClientCapability::HomeSnapshot],
                ))
                .unwrap();
            connection
                .send(&RequestEnvelope {
                    api_major: API_MAJOR,
                    message_version: MESSAGE_VERSION,
                    request_id: RequestId::parse("request-ipc-test").unwrap(),
                    payload: RequestPayload::LibraryGetHomeSnapshot,
                })
                .unwrap();
            connection.receive::<ResponseEnvelope>().unwrap()
        });

        let policy = HandshakePolicy::new(secret, vec![ClientCapability::HomeSnapshot]);
        let mut connection = server.accept_authenticated(&policy).unwrap();
        let request = connection.receive_request().unwrap();
        connection
            .send_response(&ResponseEnvelope::success(
                request.request_id,
                ResponsePayload::Accepted,
            ))
            .unwrap();

        let response = client.join().unwrap();
        assert!(response.is_valid());
        assert_eq!(response.result, Some(ResponsePayload::Accepted));
    }

    #[test]
    fn read_only_client_is_denied_over_the_real_transport() {
        let endpoint = endpoint();
        let server = IpcServer::bind(&endpoint).unwrap();
        let secret = EphemeralSecret::new([9; EPHEMERAL_SECRET_BYTES]);
        let client_endpoint = endpoint.clone();
        let client_secret = secret.clone();

        let client = thread::spawn(move || {
            let mut connection = IpcConnection::connect(&client_endpoint).unwrap();
            connection
                .send(&handshake(
                    client_secret,
                    vec![ClientCapability::DiagnosticsRead],
                ))
                .unwrap();
            connection
                .send(&RequestEnvelope {
                    api_major: API_MAJOR,
                    message_version: MESSAGE_VERSION,
                    request_id: RequestId::parse("request-denied-test").unwrap(),
                    payload: RequestPayload::SessionStart {
                        game_id: GameId::parse("game-test-001").unwrap(),
                    },
                })
                .unwrap();
        });

        let policy = HandshakePolicy::new(secret, vec![ClientCapability::DiagnosticsRead]);
        let mut connection = server.accept_authenticated(&policy).unwrap();
        let error = connection.receive_request().unwrap_err();
        assert!(matches!(
            error,
            IpcError::Unauthorized(AuthorizationError::MissingCapability(
                ClientCapability::SessionCommands
            ))
        ));
        client.join().unwrap();
    }
}
