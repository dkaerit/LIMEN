use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use limen_bridge_fake::{FakeBridge, FakeRuntimeMode};
use limen_core::Core;
use limen_domain::GameId;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => {
            eprintln!(
                "{{\"level\":\"error\",\"module\":\"core\",\"code\":\"{code}\",\"message\":\"The simulated session could not be completed.\"}}"
            );
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), &'static str> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = match arguments.as_slice() {
        [flag] if flag == "--self-check" => FakeRuntimeMode::Normal,
        [flag, mode] if flag == "--simulate" => match mode.as_str() {
            "normal" => FakeRuntimeMode::Normal,
            "crash" => FakeRuntimeMode::Crash,
            "timeout" => FakeRuntimeMode::Hang,
            _ => return Err("CORE_INVALID_SIMULATION_MODE"),
        },
        _ => return Err("CORE_USAGE"),
    };

    let executable = fake_runtime_path().map_err(|_| "CORE_RUNTIME_PATH")?;
    let bridge = FakeBridge::new(executable, mode);
    let mut core = Core::default();
    let timeout = if mode == FakeRuntimeMode::Hang {
        Duration::from_millis(100)
    } else {
        Duration::from_secs(2)
    };
    let snapshot = core
        .run_session(
            &bridge,
            GameId::parse("game-placeholder-001").map_err(|_| "CORE_GAME_ID")?,
            timeout,
            &AtomicBool::new(false),
        )
        .map_err(|_| "CORE_SIMULATION_FAILED")?;

    println!(
        "{{\"level\":\"info\",\"module\":\"core\",\"code\":\"CORE_SESSION_RESULT\",\"session_id\":\"{}\",\"state\":\"{}\",\"outcome\":\"{}\",\"sequence\":{}}}",
        snapshot.session_id,
        snapshot.state.as_str(),
        snapshot.outcome.map_or("none", |outcome| outcome.as_str()),
        snapshot.last_sequence
    );
    Ok(())
}

fn fake_runtime_path() -> Result<PathBuf, std::io::Error> {
    let mut path = std::env::current_exe()?;
    path.set_file_name(format!(
        "limen-fake-runtime{}",
        std::env::consts::EXE_SUFFIX
    ));
    Ok(path)
}
