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
pub struct SourceOptions {
    ignore: Vec<String>,
}
