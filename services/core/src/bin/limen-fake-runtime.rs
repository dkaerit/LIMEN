use std::process::ExitCode;
use std::time::Duration;

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mut mode = None;
    let mut game_id = None;
    let mut index = 0;

    while index < arguments.len() {
        match arguments[index].as_str() {
            "--mode" if index + 1 < arguments.len() => {
                mode = Some(arguments[index + 1].as_str());
                index += 2;
            }
            "--game-id" if index + 1 < arguments.len() => {
                game_id = Some(arguments[index + 1].as_str());
                index += 2;
            }
            _ => return ExitCode::from(2),
        }
    }

    if game_id.is_none() {
        return ExitCode::from(2);
    }

    match mode {
        Some("normal") => {
            std::thread::sleep(Duration::from_millis(35));
            ExitCode::SUCCESS
        }
        Some("crash") => ExitCode::from(23),
        Some("hang") => {
            std::thread::sleep(Duration::from_secs(60));
            ExitCode::SUCCESS
        }
        _ => ExitCode::from(2),
    }
}
