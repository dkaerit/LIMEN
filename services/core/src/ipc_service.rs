use std::error::Error;
use std::fmt;

use limen_contracts::RequestPayload;
use limen_transport::{EndpointName, HandshakePolicy, IpcError, IpcServer};

use crate::{CoreApi, CoreError};

pub struct CoreIpcService {
    server: IpcServer,
    policy: HandshakePolicy,
    api: CoreApi,
}

impl CoreIpcService {
    pub fn bind(
        endpoint: &EndpointName,
        policy: HandshakePolicy,
        api: CoreApi,
    ) -> Result<Self, CoreIpcServiceError> {
        Ok(Self {
            server: IpcServer::bind(endpoint)?,
            policy,
            api,
        })
    }

    pub fn serve_next(&self) -> Result<(), CoreIpcServiceError> {
        let mut connection = self.server.accept_authenticated(&self.policy)?;
        let request = connection.receive_request()?;
        let after_sequence = match &request.payload {
            RequestPayload::EventsSubscribe { after_sequence } => Some(*after_sequence),
            _ => None,
        };
        let response = self.api.dispatch(request);
        let response_ok = response.ok;
        connection.send_response(&response)?;

        if response_ok {
            if let Some(after_sequence) = after_sequence {
                for event in self.api.event_envelopes_after(after_sequence)? {
                    connection.send_event(&event)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum CoreIpcServiceError {
    Ipc(IpcError),
    Core(CoreError),
}

impl fmt::Display for CoreIpcServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipc(error) => error.fmt(formatter),
            Self::Core(error) => error.fmt(formatter),
        }
    }
}

impl Error for CoreIpcServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Ipc(error) => Some(error),
            Self::Core(error) => Some(error),
        }
    }
}

impl From<IpcError> for CoreIpcServiceError {
    fn from(error: IpcError) -> Self {
        Self::Ipc(error)
    }
}

impl From<CoreError> for CoreIpcServiceError {
    fn from(error: CoreError) -> Self {
        Self::Core(error)
    }
}
