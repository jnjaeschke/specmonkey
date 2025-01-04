use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::fmt;
use std::io;

/// Custom error type for SpecMonkey.
#[derive(Debug)]
pub enum SpecMonkeyError {
    IoError(io::Error),
    SerdeError(serde_yaml::Error),
    // Add more variants as needed.
}

impl fmt::Display for SpecMonkeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecMonkeyError::IoError(e) => write!(f, "IO Error: {}", e),
            SpecMonkeyError::SerdeError(e) => write!(f, "Serialization Error: {}", e),
            // Handle additional variants here.
        }
    }
}

impl std::error::Error for SpecMonkeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SpecMonkeyError::IoError(e) => Some(e),
            SpecMonkeyError::SerdeError(e) => Some(e),
            // Return sources for additional variants here.
        }
    }
}

// Implement conversion from io::Error to SpecMonkeyError.
impl From<io::Error> for SpecMonkeyError {
    fn from(error: io::Error) -> Self {
        SpecMonkeyError::IoError(error)
    }
}

// Implement conversion from serde_yaml::Error to SpecMonkeyError.
impl From<serde_yaml::Error> for SpecMonkeyError {
    fn from(error: serde_yaml::Error) -> Self {
        SpecMonkeyError::SerdeError(error)
    }
}

// Implement conversion from SpecMonkeyError to PyErr.
impl From<SpecMonkeyError> for PyErr {
    fn from(error: SpecMonkeyError) -> PyErr {
        match error {
            SpecMonkeyError::IoError(e) => PyIOError::new_err(e.to_string()),
            SpecMonkeyError::SerdeError(e) => PyValueError::new_err(e.to_string()),
            // Handle additional variants here, mapping to appropriate PyErr types.
        }
    }
}
