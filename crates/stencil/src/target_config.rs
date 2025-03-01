// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use stencil_error::StencilError;

#[derive(Debug, Deserialize, Serialize)]
pub struct TargetConfig {
    pub stencil: ConfigStencil,
    pub project: ConfigProject,
    pub arguments: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigStencil {
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigProject {
    pub name: String,
    pub src: String, // TODO: should this be a Path?
}

impl TargetConfig {
    fn validate(&self) -> Result<(), StencilError> {
        if self.stencil.version.is_empty() {
            return Err(StencilError::ConfigValidation(
                "stencil.version is required".to_string(),
            ));
        }
        if self.project.name.is_empty() {
            return Err(StencilError::ConfigValidation(
                "project.name is required".to_string(),
            ));
        }
        if self.project.src.is_empty() {
            return Err(StencilError::ConfigValidation(
                "project.src is required".to_string(),
            ));
        }
        Ok(())
    }

    pub fn apply_overrides(&mut self, overrides: Vec<String>) -> Result<(), StencilError> {
        let mut override_map: HashMap<String, String> = HashMap::new();
        for override_str in overrides {
            if let Some((key, value)) = override_str.split_once('=') {
                override_map.insert(key.to_string(), value.to_string());
            } else {
                return Err(StencilError::ConfigOverride(override_str));
            }
        }

        for (key, value) in override_map {
            match key.as_str() {
                "stencil.version" => self.stencil.version = value,
                "project.name" => self.project.name = value,
                "project.src" => self.project.src = value,
                _ => {
                    if key.starts_with("arguments.") {
                        let arg_key = key.trim_start_matches("arguments.").to_string();
                        self.arguments.insert(arg_key, value);
                    } else {
                        eprintln!("Unknown override key: {key}");
                    }
                }
            }
        }

        Ok(())
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), StencilError> {
        let contents = match toml::to_string(self) {
            Ok(contents) => contents,
            Err(e) => return Err(StencilError::from(e)),
        };
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, StencilError> {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) => return Err(StencilError::from(e)),
        };
        let config: Self = match toml::from_str(contents.as_str()) {
            Ok(config) => config,
            Err(e) => return Err(StencilError::from(e)),
        };
        match config.validate() {
            Ok(()) => Ok(config),
            Err(e) => Err(e),
        }
    }
}
