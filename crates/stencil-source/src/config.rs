// Copyright (c) 2024-2025 David Stanek <dstanek@dstanek.com>

// TODO: support for a source configuration need to be added.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceConfig {
    pub stencil: SourceStencil,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceStencil {
    pub author_name: String,
    pub author_email: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceQuestons {
    pub questions: Vec<SourceQuestion>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceQuestion {
    pub variable: String,
    pub question: String,
    pub datatype: String, // TODO: constrain to a set of known types
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceOptions {
    ignore: Vec<String>,
}
