use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::error::StencilError;

#[derive(Debug, Deserialize, Serialize)]
pub struct TargetConfig {
    pub stencil: ConfigStencil,
    pub project: ConfigProject,
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
                _ => eprintln!("Unknown override key: {}", key),
            }
        }

        Ok(())
    }

    pub fn save(&self, path: &PathBuf, config: &TargetConfig) -> Result<(), StencilError> {
        let contents = toml::to_string(config).unwrap();
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<TargetConfig, StencilError> {
        let contents = fs::read_to_string(path)?;
        let config: TargetConfig = toml::from_str(contents.as_str()).unwrap();
        match config.validate() {
            Ok(_) => Ok(config),
            Err(e) => Err(e),
        }
    }
}
