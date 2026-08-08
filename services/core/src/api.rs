use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use limen_bridge_fake::{FakeBridge, FakeRuntimeMode};
use limen_contracts::{
    API_MAJOR, MESSAGE_VERSION, ApiError, ApiErrorCode, EventEnvelope, EventPayload, ModuleHealth,
    ModuleStatus, RequestEnvelope, RequestPayload, ResponseEnvelope, ResponsePayload, SystemInfo,
};
use limen_domain::SessionId;

use crate::{Core, CoreError};

#[derive(Clone, Debug)]
pub struct SimulatedSessionConfig {
    pub runtime_executable: PathBuf,
    pub mode: FakeRuntimeMode,
    pub timeout: Duration,
}

#[derive(Clone, Debug)]
pub struct CoreApi {
    core: Core,
    simulated_session: SimulatedSessionConfig,
    cancellation: Arc<Mutex<Option<ActiveCancellation>>>,
}

#[derive(Debug)]
struct ActiveCancellation {
    session_id: SessionId,
    signal: Arc<AtomicBool>,
}

impl CoreApi {
    pub fn new(core: Core, simulated_session: SimulatedSessionConfig) -> Self {
        Self {
            core,
            simulated_session,
            cancellation: Arc::new(Mutex::new(None)),
        }
    }

    pub fn core(&self) -> &Core {
        &self.core
    }

    pub fn dispatch(&self, request: RequestEnvelope) -> ResponseEnvelope {
        if let Err(error) = request.compatibility().validate() {
            return ResponseEnvelope::failure(
                request.request_id,
                ApiError {
                    code: ApiErrorCode::InvalidRequest,
                    user_message: error.to_string(),
                    retryable: false,
                },
            );
        }

        let request_id = request.request_id;
        let result = self.dispatch_payload(request.payload);
        match result {
            Ok(payload) => ResponseEnvelope::success(request_id, payload),
            Err(error) => ResponseEnvelope::failure(request_id, api_error(error)),
        }
    }

    pub fn event_envelopes_after(
        &self,
        sequence: u64,
    ) -> Result<Vec<EventEnvelope>, CoreError> {
        self.core
            .events_after(sequence)
            .map(|events| events.into_iter().map(event_envelope).collect())
    }

    fn dispatch_payload(&self, request: RequestPayload) -> Result<ResponsePayload, CoreError> {
        match request {
            RequestPayload::SystemGetInfo => Ok(ResponsePayload::SystemInfo(SystemInfo {
                core_version: env!("CARGO_PKG_VERSION").to_owned(),
                api_major: API_MAJOR,
                modules: vec![
                    module("core"),
                    module("transport"),
                    module("bridge.fake"),
                ],
            })),
            RequestPayload::LibraryGetHomeSnapshot => self
                .core
                .home_snapshot()
                .map(ResponsePayload::HomeSnapshot),
            RequestPayload::SessionStart { game_id } => self.start_session(game_id),
            RequestPayload::SessionGet { session_id } => self
                .core
                .session_snapshot_for(&session_id)?
                .map(ResponsePayload::Session)
                .ok_or(CoreError::SessionNotFound),
            RequestPayload::SessionStop { session_id } => self.stop_session(&session_id),
            RequestPayload::EventsSubscribe { .. } => self
                .core
                .last_event_sequence()
                .map(|current_sequence| ResponsePayload::EventsSubscribed { current_sequence }),
        }
    }

    fn start_session(
        &self,
        game_id: limen_domain::GameId,
    ) -> Result<ResponsePayload, CoreError> {
        let session_id = self.core.start_session(game_id.clone())?;
        let signal = Arc::new(AtomicBool::new(false));
        {
            let mut cancellation = self.lock_cancellation()?;
            *cancellation = Some(ActiveCancellation {
                session_id: session_id.clone(),
                signal: Arc::clone(&signal),
            });
        }

        let initial_snapshot = self
            .core
            .session_snapshot_for(&session_id)?
            .ok_or(CoreError::SessionNotFound)?;
        let core = self.core.clone();
        let cancellation = Arc::clone(&self.cancellation);
        let config = self.simulated_session.clone();
        let worker_session_id = session_id.clone();
        let _worker = std::thread::spawn(move || {
            let bridge = FakeBridge::new(config.runtime_executable, config.mode);
            let _ = core.continue_session(
                &bridge,
                &worker_session_id,
                game_id,
                config.timeout,
                signal.as_ref(),
            );

            if let Ok(mut active) = cancellation.lock() {
                if active
                    .as_ref()
                    .is_some_and(|entry| entry.session_id == worker_session_id)
                {
                    *active = None;
                }
            }
        });

        Ok(ResponsePayload::Session(initial_snapshot))
    }

    fn stop_session(&self, session_id: &SessionId) -> Result<ResponsePayload, CoreError> {
        let cancellation = self.lock_cancellation()?;
        let active = cancellation
            .as_ref()
            .filter(|entry| &entry.session_id == session_id)
            .ok_or(CoreError::SessionNotFound)?;
        active.signal.store(true, Ordering::Release);
        Ok(ResponsePayload::Accepted)
    }

    fn lock_cancellation(
        &self,
    ) -> Result<MutexGuard<'_, Option<ActiveCancellation>>, CoreError> {
        self.cancellation
            .lock()
            .map_err(|_| CoreError::StatePoisoned)
    }
}

fn module(name: &str) -> ModuleStatus {
    ModuleStatus {
        module: name.to_owned(),
        state: ModuleHealth::Ready,
    }
}

fn event_envelope(event: limen_session::SessionEvent) -> EventEnvelope {
    EventEnvelope {
        api_major: API_MAJOR,
        message_version: MESSAGE_VERSION,
        sequence: event.sequence,
        session_id: Some(event.session_id),
        payload: EventPayload::SessionStateChanged {
            previous: event.previous,
            current: event.current,
        },
    }
}

fn api_error(error: CoreError) -> ApiError {
    let (code, user_message, retryable) = match error {
        CoreError::SessionAlreadyActive => (
            ApiErrorCode::SessionAlreadyActive,
            "Ya hay una sesión activa.",
            false,
        ),
        CoreError::SessionNotFound => (
            ApiErrorCode::SessionNotFound,
            "No se ha encontrado esa sesión.",
            false,
        ),
        CoreError::InvalidTransition(_) => (
            ApiErrorCode::InvalidSessionTransition,
            "La sesión no puede realizar esa transición.",
            false,
        ),
        CoreError::BridgeValidationFailed | CoreError::BridgePlanFailed => (
            ApiErrorCode::InvalidRequest,
            "El runtime de prueba no está disponible para esta solicitud.",
            false,
        ),
        CoreError::ProcessStartFailed | CoreError::ProcessIo(_) => (
            ApiErrorCode::Internal,
            "No se pudo iniciar o supervisar el runtime de prueba.",
            true,
        ),
        CoreError::StatePoisoned | CoreError::InvalidIdentifier(_) => (
            ApiErrorCode::Internal,
            "Core no pudo completar la operación de forma segura.",
            true,
        ),
    };

    ApiError {
        code,
        user_message: user_message.to_owned(),
        retryable,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use limen_contracts::{Compatibility, RequestPayload};
    use limen_domain::{GameId, RequestId};

    use super::*;

    static REQUEST: AtomicU64 = AtomicU64::new(1);

    fn request(payload: RequestPayload) -> RequestEnvelope {
        let sequence = REQUEST.fetch_add(1, Ordering::Relaxed);
        RequestEnvelope {
            api_major: Compatibility::CURRENT.api_major,
            message_version: Compatibility::CURRENT.message_version,
            request_id: RequestId::parse(format!("request-api-{sequence}")).unwrap(),
            payload,
        }
    }

    fn api() -> CoreApi {
        CoreApi::new(
            Core::default(),
            SimulatedSessionConfig {
                runtime_executable: PathBuf::from("unused-in-query-tests"),
                mode: FakeRuntimeMode::Normal,
                timeout: Duration::from_secs(1),
            },
        )
    }

    #[test]
    fn query_requests_return_typed_snapshots() {
        let response = api().dispatch(request(RequestPayload::LibraryGetHomeSnapshot));

        assert!(response.is_valid());
        assert!(matches!(
            response.result,
            Some(ResponsePayload::HomeSnapshot(_))
        ));
    }

    #[test]
    fn incompatible_request_fails_closed() {
        let mut invalid = request(RequestPayload::SessionStart {
            game_id: GameId::parse("game-test-001").unwrap(),
        });
        invalid.api_major += 1;
        let response = api().dispatch(invalid);

        assert_eq!(
            response.error.map(|error| error.code),
            Some(ApiErrorCode::InvalidRequest)
        );
    }
}
