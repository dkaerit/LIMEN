use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use limen_bridge_fake::FakeRuntimeMode;
use limen_contracts::{
    API_MAJOR, ClientCapability, ClientChannel, Compatibility, EPHEMERAL_SECRET_BYTES,
    EphemeralSecret, EventEnvelope, EventPayload, HandshakeRequest, MESSAGE_VERSION,
    RequestEnvelope, RequestPayload, ResponseEnvelope, ResponsePayload,
};
use limen_core::{Core, CoreApi, CoreIpcService, CoreIpcServiceError, SimulatedSessionConfig};
use limen_domain::{ClientId, GameId, RequestId, SessionOutcome, SessionState};
use limen_transport::{AuthorizationError, EndpointName, HandshakePolicy, IpcConnection, IpcError};

static NEXT_ENDPOINT: AtomicU64 = AtomicU64::new(1);
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

fn fake_runtime() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_limen-fake-runtime"))
}

fn endpoint(role: &str) -> EndpointName {
    let sequence = NEXT_ENDPOINT.fetch_add(1, Ordering::Relaxed);
    EndpointName::parse(format!(
        "limen-core-{role}-{}-{sequence}",
        std::process::id()
    ))
    .unwrap()
}

fn request(payload: RequestPayload) -> RequestEnvelope {
    let sequence = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
    RequestEnvelope {
        api_major: API_MAJOR,
        message_version: MESSAGE_VERSION,
        request_id: RequestId::parse(format!("request-ipc-core-{sequence}")).unwrap(),
        payload,
    }
}

fn handshake(
    secret: EphemeralSecret,
    channel: ClientChannel,
    capabilities: Vec<ClientCapability>,
) -> HandshakeRequest {
    HandshakeRequest {
        api_major: Compatibility::CURRENT.api_major,
        message_version: Compatibility::CURRENT.message_version,
        client_id: ClientId::parse("client-core-integration").unwrap(),
        client_name: "Core integration client".to_owned(),
        channel,
        capabilities,
        secret,
    }
}

fn exchange(
    endpoint: EndpointName,
    secret: EphemeralSecret,
    capabilities: Vec<ClientCapability>,
    payload: RequestPayload,
) -> thread::JoinHandle<ResponseEnvelope> {
    thread::spawn(move || {
        let mut connection = IpcConnection::connect(&endpoint).unwrap();
        connection
            .send(&handshake(secret, ClientChannel::Commands, capabilities))
            .unwrap();
        connection.send(&request(payload)).unwrap();
        connection.receive().unwrap()
    })
}

fn wait_for_state(core: &Core, expected: SessionState) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let state = core
            .home_snapshot()
            .unwrap()
            .active_session
            .map(|session| session.state);
        if state == Some(expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "session did not reach {expected:?}"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn home_can_disconnect_reconnect_and_runtime_console_only_observes() {
    let core = Core::default();
    let api = CoreApi::new(
        core.clone(),
        SimulatedSessionConfig {
            runtime_executable: fake_runtime(),
            mode: FakeRuntimeMode::Hang,
            timeout: Duration::from_secs(5),
        },
    );

    let home_endpoint = endpoint("home");
    let home_secret = EphemeralSecret::new([21; EPHEMERAL_SECRET_BYTES]);
    let home_policy = HandshakePolicy::new(
        home_secret.clone(),
        vec![
            ClientCapability::HomeSnapshot,
            ClientCapability::SessionCommands,
        ],
    );
    let home_service = CoreIpcService::bind(&home_endpoint, home_policy, api.clone()).unwrap();

    let start_client = exchange(
        home_endpoint.clone(),
        home_secret.clone(),
        vec![ClientCapability::SessionCommands],
        RequestPayload::SessionStart {
            game_id: GameId::parse("game-placeholder-001").unwrap(),
        },
    );
    home_service.serve_next().unwrap();
    let start_response = start_client.join().unwrap();
    let started_session = match start_response.result {
        Some(ResponsePayload::Session(snapshot)) => snapshot.session_id,
        other => panic!("unexpected start response: {other:?}"),
    };

    wait_for_state(&core, SessionState::Running);

    let reconnecting_home = exchange(
        home_endpoint.clone(),
        home_secret.clone(),
        vec![ClientCapability::HomeSnapshot],
        RequestPayload::LibraryGetHomeSnapshot,
    );
    home_service.serve_next().unwrap();
    let snapshot_response = reconnecting_home.join().unwrap();
    let reconnected_snapshot = match snapshot_response.result {
        Some(ResponsePayload::HomeSnapshot(snapshot)) => snapshot,
        other => panic!("unexpected Home response: {other:?}"),
    };
    assert_eq!(
        reconnected_snapshot
            .active_session
            .as_ref()
            .map(|session| &session.session_id),
        Some(&started_session)
    );
    assert_eq!(
        reconnected_snapshot
            .active_session
            .as_ref()
            .map(|session| session.state),
        Some(SessionState::Running)
    );

    let stop_client = exchange(
        home_endpoint,
        home_secret,
        vec![ClientCapability::SessionCommands],
        RequestPayload::SessionStop {
            session_id: started_session.clone(),
        },
    );
    home_service.serve_next().unwrap();
    assert_eq!(
        stop_client.join().unwrap().result,
        Some(ResponsePayload::Accepted)
    );
    wait_for_state(&core, SessionState::Cancelled);
    assert_eq!(
        core.home_snapshot()
            .unwrap()
            .active_session
            .and_then(|session| session.outcome),
        Some(SessionOutcome::Cancelled)
    );

    let console_endpoint = endpoint("console-events");
    let console_secret = EphemeralSecret::new([22; EPHEMERAL_SECRET_BYTES]);
    let console_service = CoreIpcService::bind(
        &console_endpoint,
        HandshakePolicy::new(
            console_secret.clone(),
            vec![ClientCapability::DiagnosticsRead],
        ),
        api.clone(),
    )
    .unwrap();
    let event_client = thread::spawn(move || {
        let mut connection = IpcConnection::connect(&console_endpoint).unwrap();
        connection
            .send(&handshake(
                console_secret,
                ClientChannel::Events,
                vec![ClientCapability::DiagnosticsRead],
            ))
            .unwrap();
        connection
            .send(&request(RequestPayload::EventsSubscribe {
                after_sequence: 0,
            }))
            .unwrap();
        let response: ResponseEnvelope = connection.receive().unwrap();
        let current_sequence = match response.result {
            Some(ResponsePayload::EventsSubscribed { current_sequence }) => current_sequence,
            other => panic!("unexpected subscription response: {other:?}"),
        };
        let mut events = Vec::new();
        while events
            .last()
            .map_or(0, |event: &EventEnvelope| event.sequence)
            < current_sequence
        {
            events.push(connection.receive::<EventEnvelope>().unwrap());
        }
        events
    });
    console_service.serve_next().unwrap();
    let events = event_client.join().unwrap();
    assert!(events.iter().any(|event| {
        matches!(
            &event.payload,
            EventPayload::SessionStateChanged {
                current: SessionState::Running,
                ..
            }
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            &event.payload,
            EventPayload::SessionStateChanged {
                current: SessionState::Cancelled,
                ..
            }
        )
    }));

    let denied_endpoint = endpoint("console-denied");
    let denied_secret = EphemeralSecret::new([23; EPHEMERAL_SECRET_BYTES]);
    let denied_service = CoreIpcService::bind(
        &denied_endpoint,
        HandshakePolicy::new(
            denied_secret.clone(),
            vec![ClientCapability::DiagnosticsRead],
        ),
        api,
    )
    .unwrap();
    let denied_client = thread::spawn(move || {
        let mut connection = IpcConnection::connect(&denied_endpoint).unwrap();
        connection
            .send(&handshake(
                denied_secret,
                ClientChannel::Commands,
                vec![ClientCapability::DiagnosticsRead],
            ))
            .unwrap();
        connection
            .send(&request(RequestPayload::SessionStart {
                game_id: GameId::parse("game-placeholder-002").unwrap(),
            }))
            .unwrap();
    });
    let error = denied_service.serve_next().unwrap_err();
    denied_client.join().unwrap();
    assert!(matches!(
        error,
        CoreIpcServiceError::Ipc(IpcError::Unauthorized(
            AuthorizationError::MissingCapability(ClientCapability::SessionCommands)
        ))
    ));
}
