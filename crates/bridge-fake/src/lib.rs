#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use limen_bridge_sdk::{
    Bridge, BridgeCapability, BridgeDescriptor, InvalidLaunchPlan, LaunchIntent, LaunchPlan,
};

const CAPABILITIES: &[BridgeCapability] = &[
    BridgeCapability::Fullscreen,
    BridgeCapability::NoGui,
    BridgeCapability::OrderedStop,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeRuntimeMode {
    Normal,
    Crash,
    Hang,
}

impl FakeRuntimeMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Crash => "crash",
            Self::Hang => "hang",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FakeBridge {
    executable: PathBuf,
    mode: FakeRuntimeMode,
}

impl FakeBridge {
    pub fn new(executable: PathBuf, mode: FakeRuntimeMode) -> Self {
        Self { executable, mode }
    }
}

impl Bridge for FakeBridge {
    type Error = FakeBridgeError;

    fn descriptor(&self) -> BridgeDescriptor {
        BridgeDescriptor {
            id: "bridge.fake",
            version: env!("CARGO_PKG_VERSION"),
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, _intent: &LaunchIntent) -> Result<(), Self::Error> {
        if !self.executable.is_absolute() {
            return Err(FakeBridgeError::ExecutableUnavailable);
        }
        Ok(())
    }

    fn plan_launch(&self, intent: &LaunchIntent) -> Result<LaunchPlan, Self::Error> {
        self.validate(intent)?;
        LaunchPlan::new(
            self.executable.clone(),
            vec![
                OsString::from("--mode"),
                OsString::from(self.mode.as_str()),
                OsString::from("--game-id"),
                OsString::from(intent.game_id.as_str()),
            ],
            BTreeMap::new(),
        )
        .map_err(FakeBridgeError::InvalidPlan)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeBridgeError {
    ExecutableUnavailable,
    InvalidPlan(InvalidLaunchPlan),
}

impl fmt::Display for FakeBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecutableUnavailable => formatter.write_str("fake runtime is unavailable"),
            Self::InvalidPlan(error) => error.fmt(formatter),
        }
    }
}

impl Error for FakeBridgeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use limen_bridge_sdk::Bridge;
    use limen_domain::GameId;

    #[test]
    fn fake_bridge_builds_structured_argv_without_a_shell() {
        let bridge = FakeBridge::new(std::env::current_exe().unwrap(), FakeRuntimeMode::Normal);
        let plan = bridge
            .plan_launch(&LaunchIntent {
                game_id: GameId::parse("game-placeholder-001").unwrap(),
            })
            .unwrap();

        assert_eq!(plan.executable(), std::env::current_exe().unwrap());
        assert_eq!(plan.argv().len(), 4);
        assert_eq!(plan.argv()[0].as_os_str(), std::ffi::OsStr::new("--mode"));
        assert_eq!(
            plan.argv()[2].as_os_str(),
            std::ffi::OsStr::new("--game-id")
        );
    }
}
