use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum OrchestrionError {
    IoError(std::io::Error),
    StrError(String),
}

impl From<std::io::Error> for OrchestrionError {
    fn from(e: std::io::Error) -> Self {
        OrchestrionError::IoError(e)
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
            OrchestrionError::IoError(e) => write!(f, "IO error: {e}"),
            OrchestrionError::StrError(s) => write!(f, "String error: {s}"),
        }
    }
}
