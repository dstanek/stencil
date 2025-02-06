// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use std::fs;
use std::path::PathBuf;

use crate::error::StencilError;

pub trait RenderableIterator: Iterator<Item = Result<Renderable, StencilError>> {}
impl<T> RenderableIterator for T where T: Iterator<Item = Result<Renderable, StencilError>> {}

pub struct File {
    pub relative_path: String,
    pub content: String,
}

impl File {
    pub fn new(relative_path: String, content: String) -> Self {
        File {
            relative_path,
            content,
        }
    }

    pub fn from_path(
        relative_path: String,
        fully_qualified_path: &PathBuf,
    ) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(fully_qualified_path)?;
        Ok(File {
            relative_path,
            content,
        })
    }

    pub fn empty() -> Self {
        File {
            relative_path: "/dev/null".to_string(),
            content: "".to_string(),
        }
    }
}

pub struct Directory {
    pub relative_path: String,
}

impl Directory {
    pub fn new(relative_path: String) -> Self {
        Directory { relative_path }
    }
}

pub enum Renderable {
    File(File),
    Directory(Directory),
}
