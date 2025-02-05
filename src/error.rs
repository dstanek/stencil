// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use std::convert::From;
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
    #[error("Deserialization error:{0}")]
    TomlDeserialization(#[from] toml::de::Error),

    #[error("Serialization error: {0}")]
    TomlSerialization(#[from] toml::ser::Error),

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

impl From<io::Error> for StencilError {
    fn from(error: io::Error) -> Self {
        StencilError::Io(error)
    }
}
