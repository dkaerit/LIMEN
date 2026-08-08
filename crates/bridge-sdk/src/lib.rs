#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};

use limen_domain::GameId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeCapability {
    Fullscreen,
    NoGui,
    OrderedStop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeDescriptor {
    pub id: &'static str,
    pub version: &'static str,
    pub capabilities: &'static [BridgeCapability],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchIntent {
    pub game_id: GameId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchPlan {
    executable: PathBuf,
    argv: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
}

impl LaunchPlan {
    pub fn new(
        executable: impl Into<PathBuf>,
        argv: Vec<OsString>,
        environment: BTreeMap<OsString, OsString>,
    ) -> Result<Self, InvalidLaunchPlan> {
        let executable = executable.into();
        if !executable.is_absolute() || executable.as_os_str().is_empty() {
            return Err(InvalidLaunchPlan::ExecutableMustBeAbsolute);
        }
        if argv.iter().any(|argument| argument.is_empty()) {
            return Err(InvalidLaunchPlan::EmptyArgument);
        }
        if environment.keys().any(|key| key.is_empty()) {
            return Err(InvalidLaunchPlan::EmptyEnvironmentKey);
        }

        Ok(Self {
            executable,
            argv,
            environment,
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn argv(&self) -> &[OsString] {
        &self.argv
    }

    pub fn environment(&self) -> &BTreeMap<OsString, OsString> {
        &self.environment
    }

    pub fn redacted_summary(&self) -> RedactedLaunchPlan<'_> {
        RedactedLaunchPlan(self)
    }
}

pub struct RedactedLaunchPlan<'a>(&'a LaunchPlan);

impl fmt::Debug for RedactedLaunchPlan<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LaunchPlan")
            .field(
                "executable",
                &self
                    .0
                    .executable
                    .file_name()
                    .unwrap_or(OsStr::new("[unknown]")),
            )
            .field("argument_count", &self.0.argv.len())
            .field(
                "environment_keys",
                &self.0.environment.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidLaunchPlan {
    ExecutableMustBeAbsolute,
    EmptyArgument,
    EmptyEnvironmentKey,
}

impl fmt::Display for InvalidLaunchPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ExecutableMustBeAbsolute => "launch executable must be an absolute path",
            Self::EmptyArgument => "launch arguments must not be empty",
            Self::EmptyEnvironmentKey => "launch environment keys must not be empty",
        };
        formatter.write_str(message)
    }
}

impl Error for InvalidLaunchPlan {}

pub trait Bridge {
    type Error: Error + Send + Sync + 'static;

    fn descriptor(&self) -> BridgeDescriptor;
    fn validate(&self, intent: &LaunchIntent) -> Result<(), Self::Error>;
    fn plan_launch(&self, intent: &LaunchIntent) -> Result<LaunchPlan, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_plan_rejects_relative_executables() {
        let result = LaunchPlan::new("relative/runtime", Vec::new(), BTreeMap::new());
        assert_eq!(result, Err(InvalidLaunchPlan::ExecutableMustBeAbsolute));
    }

    #[test]
    fn debug_summary_never_contains_the_full_executable_path_or_arguments() {
        let executable = std::env::current_exe().unwrap();
        let secret_argument = OsString::from("private-game-path");
        let plan = LaunchPlan::new(
            executable,
            vec![secret_argument],
            BTreeMap::from([(OsString::from("LIMEN_MODE"), OsString::from("test"))]),
        )
        .unwrap();
        let summary = format!("{:?}", plan.redacted_summary());

        assert!(!summary.contains("private-game-path"));
        assert!(summary.contains("argument_count"));
    }
}
