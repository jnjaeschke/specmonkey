use std::fmt;
use std::io;

/// Custom error type for SpecMonkey.
#[derive(Debug)]
pub enum SpecMonkeyError {
    Error(String),
    IoError(io::Error),
    SerdeYamlError(serde_yaml::Error),
    SerdeJsonError(serde_json::Error),
    // Add more variants as needed.
}

impl fmt::Display for SpecMonkeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecMonkeyError::Error(e) => write!(f, "Error: {}", e),
            SpecMonkeyError::IoError(e) => write!(f, "IO Error: {}", e),
            SpecMonkeyError::SerdeYamlError(e) => write!(f, "Yaml Serialization Error: {}", e),
            SpecMonkeyError::SerdeJsonError(e) => write!(f, "JSON Serialization Error: {}", e),
            // Handle additional variants here.
        }
    }
}

impl std::error::Error for SpecMonkeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SpecMonkeyError::Error(_) => None,
            SpecMonkeyError::IoError(e) => Some(e),
            SpecMonkeyError::SerdeYamlError(e) => Some(e),
            SpecMonkeyError::SerdeJsonError(e) => Some(e),
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
        SpecMonkeyError::SerdeYamlError(error)
    }
}
// Implement conversion from serde_yaml::Error to SpecMonkeyError.
impl From<serde_json::Error> for SpecMonkeyError {
    fn from(error: serde_json::Error) -> Self {
        SpecMonkeyError::SerdeJsonError(error)
    }
}
