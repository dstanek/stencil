use std::path::PathBuf;

use crate::error::StencilError;
use crate::source::filesystem::FilesystemIterator;
use crate::source::git::GithubRepoIterator;
use crate::source::model::Renderable;

pub fn renderables(source: &String) -> Result<Vec<Renderable>, StencilError> {
    // If the config.project.src starts with git:// or github:// return a GithubRepoIterator otherwise return a Filesystem iterator
    let iterator = match source.starts_with("github://") {
        true => {
            // split the string by /
            let source = source.split("://").nth(1).unwrap_or(source);
            let parts: Vec<&str> = source.split('/').collect();
            let stencil_path = PathBuf::from(source);
            GithubRepoIterator::new(
                parts[0].to_string(),
                parts[1].to_string(),
                String::from("stencil"),
            )?
        }
        false => {
            let stencil_path = PathBuf::from(source);
            FilesystemIterator::new(&stencil_path)?
        }
    };

    let renderables: Vec<Renderable> = iterator.filter_map(Result::ok).collect();
    Ok(renderables)
}
