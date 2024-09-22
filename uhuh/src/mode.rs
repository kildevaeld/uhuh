use core::fmt;
use std::str::FromStr;

use vaerdi::{convert::FromValue, ConvertError, Value};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    #[serde(
        rename = "development",
        alias = "Development",
        alias = "dev",
        alias = "Dev"
    )]
    Development,
    #[serde(
        rename = "production",
        alias = "Production",
        alias = "prod",
        alias = "Prod"
    )]
    Production,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Development => f.write_str("Development"),
            Mode::Production => f.write_str("Production"),
        }
    }
}

impl FromValue for Mode {
    type Error = ModeParseErr;
    fn from_value(value: vaerdi::Value) -> Result<Self, Self::Error> {
        let Some(str) = value.as_string() else {
            return Err(ModeParseErr);
        };

        str.parse()
    }
}

impl From<Mode> for Value {
    fn from(value: Mode) -> Self {
        Value::String(value.to_string().into())
    }
}

impl From<ModeParseErr> for ConvertError {
    fn from(value: ModeParseErr) -> Self {
        ConvertError::unknown(value)
    }
}

#[derive(Debug)]
pub struct ModeParseErr;

impl fmt::Display for ModeParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid mode")
    }
}

impl std::error::Error for ModeParseErr {}

impl FromStr for Mode {
    type Err = ModeParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mode = match s.to_lowercase().as_str() {
            "development" | "dev" => Mode::Development,
            "production" | "prod" => Mode::Production,
            _ => return Err(ModeParseErr),
        };
        Ok(mode)
    }
}
