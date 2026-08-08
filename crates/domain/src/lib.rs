#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

const MAX_IDENTIFIER_BYTES: usize = 128;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier(String);

impl Identifier {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidIdentifier> {
        let value = value.into();
        let valid_length = !value.is_empty() && value.len() <= MAX_IDENTIFIER_BYTES;
        let valid_characters = value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'));

        if !valid_length || !valid_characters {
            return Err(InvalidIdentifier);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidIdentifier;

impl fmt::Display for InvalidIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("identifier must contain 1-128 portable characters")
    }
}

impl Error for InvalidIdentifier {}

macro_rules! domain_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(Identifier);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, InvalidIdentifier> {
                Identifier::parse(value).map(Self)
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

domain_id!(ClientId);
domain_id!(GameId);
domain_id!(RequestId);
domain_id!(SessionId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Requested,
    Validating,
    Preparing,
    Launching,
    Running,
    Stopping,
    Finished,
    Crashed,
    RecoveringHome,
    Failed,
    Cancelled,
    TimedOut,
}

impl SessionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Validating => "validating",
            Self::Preparing => "preparing",
            Self::Launching => "launching",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Finished => "finished",
            Self::Crashed => "crashed",
            Self::RecoveringHome => "recovering_home",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::RecoveringHome | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOutcome {
    Finished,
    Crashed,
    Failed,
    Cancelled,
    TimedOut,
}

impl SessionOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::Crashed => "crashed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_reject_paths_and_unbounded_input() {
        assert!(GameId::parse("game-ffx-placeholder").is_ok());
        assert!(GameId::parse("../games/example.iso").is_err());
        assert!(GameId::parse("a".repeat(129)).is_err());
    }

    #[test]
    fn only_completed_recovery_or_failures_are_terminal() {
        assert!(!SessionState::Running.is_terminal());
        assert!(!SessionState::Finished.is_terminal());
        assert!(SessionState::RecoveringHome.is_terminal());
        assert!(SessionState::TimedOut.is_terminal());
    }
}
