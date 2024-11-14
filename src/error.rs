use std::convert::From;
//use std::fmt;
use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StencilError {
    #[error("I/O Error: {0}")]
    Io(io::Error),

    #[error("Destination '{0}' already exists")]
    DestinationExists(String),

    #[error("{0}")]
    Other(String),

    // configuration errors
    #[error("Deserialization erorr")]
    Toml(#[from] toml::de::Error),

    #[error("Validation error: {0}")]
    ConfigValidation(String),

    #[error("Invalid override : {0}")]
    ConfigOverride(String),
}

impl StencilError {
    pub fn new(msg: &str) -> Self {
        StencilError::Other(msg.to_string())
    }
}

//impl fmt::Display for StencilError {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        match self {
//            StencilError::Io(err) => write!(f, "IO error: {}", err),
//            StencilError::Other(msg) => write!(f, "Other error: {}", msg),
//        }
//    }
//}

//impl std::error::Error for StencilError {
//    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//        match self {
//            StencilError::Io(err) => Some(err),
//            StencilError::Other(_) => None,
//        }
//    }
//}

impl From<io::Error> for StencilError {
    fn from(error: io::Error) -> Self {
        StencilError::Io(error)
    }
}
