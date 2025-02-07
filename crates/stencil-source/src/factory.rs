// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use std::path::PathBuf;

use crate::filesystem::FilesystemIterator;
use crate::git::GithubRepoIterator;
use crate::model::{Renderable, RenderableIterator};
use stencil_error::StencilError;

pub fn renderables(source: &String) -> Result<Vec<Renderable>, StencilError> {
    // TODO: maybe add Github Enterprise and Gitlab support?
    let iterator: Box<dyn RenderableIterator> = match source.starts_with("gh://") {
        true => {
            // TODO: add support for sha pinning like gh:owner/project[/path][@sha]
            let source = source.split("://").nth(1).unwrap_or(source);
            let parts: Vec<&str> = source.split('/').collect();
            let stencil_path = if parts.len() == 3 {
                parts[2].to_string()
            } else {
                "stencil".to_string()
            };
            Box::new(GithubRepoIterator::new(
                parts[0].to_string(),
                parts[1].to_string(),
                stencil_path,
            )?)
        }
        false => {
            let stencil_path = PathBuf::from(source);
            Box::new(FilesystemIterator::new(&stencil_path)?)
        }
    };

    let renderables: Vec<Renderable> = iterator.filter_map(Result::ok).collect();
    Ok(renderables)
}
