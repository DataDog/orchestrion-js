use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum OrchestrionError {
    InvalidVersionRange(String),
    IoError(std::io::Error),
    YamlParseError(yaml_rust2::ScanError),
    StrError(String),
}

impl From<std::io::Error> for OrchestrionError {
    fn from(e: std::io::Error) -> Self {
        OrchestrionError::IoError(e)
    }
}

impl From<yaml_rust2::ScanError> for OrchestrionError {
    fn from(e: yaml_rust2::ScanError) -> Self {
        OrchestrionError::YamlParseError(e)
    }
}

impl From<String> for OrchestrionError {
    fn from(s: String) -> Self {
        OrchestrionError::StrError(s)
    }
}

impl From<&str> for OrchestrionError {
    fn from(s: &str) -> Self {
        OrchestrionError::StrError(s.to_string())
    }
}

impl Display for OrchestrionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            OrchestrionError::InvalidVersionRange(s) => write!(f, "Invalid version range: {s}"),
            OrchestrionError::IoError(e) => write!(f, "IO error: {e}"),
            OrchestrionError::YamlParseError(e) => write!(f, "YAML parse error: {e}"),
            OrchestrionError::StrError(s) => write!(f, "String error: {s}"),
        }
    }
}
