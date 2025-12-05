// Copyright (c) 2024-2025 David Stanek <dstanek@dstanek.com>

use std::path::PathBuf;

use serde_json::Value;

use crate::target_config::TargetConfig;
use stencil_error::StencilError;
use stencil_rendering::Renderable as RenderableTrait;
use stencil_rendering::{render, render_str};
use stencil_source::{Directory, File, Renderable};

struct RenderableFile<'a>(&'a File);

#[allow(clippy::needless_lifetimes)]
impl<'a> RenderableTrait for RenderableFile<'a> {
    fn content(&self) -> &str {
        &self.0.content
    }
}

pub struct RenderingIterator {
    renderables: Vec<Renderable>,
    arguments: Value,
    index: usize,
}

impl Iterator for RenderingIterator {
    type Item = Result<Renderable, StencilError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.renderables.len() {
            return None;
        }

        let renderable = &self.renderables[self.index];
        self.index += 1;

        match renderable {
            Renderable::File(file) => {
                let mut dest_path =
                    PathBuf::from(render_str(&file.relative_path, &self.arguments).unwrap()); // TODO: catch bad rendering (etc...missing variable)

                if dest_path.extension().map_or(true, |ext| ext != "jinja") {
                    return Some(Ok(Renderable::File(File {
                        relative_path: dest_path.to_string_lossy().to_string(),
                        content: file.content.clone(),
                    })));
                }

                let rf = RenderableFile(file);
                let content = render(&rf, &rf, &self.arguments).unwrap();

                dest_path.set_extension("");
                Some(Ok(Renderable::File(File {
                    relative_path: dest_path.to_string_lossy().to_string(),
                    content,
                })))
            }
            Renderable::Directory(directory) => {
                Some(Ok(Renderable::Directory(Directory {
                    relative_path: render_str(&directory.relative_path, &self.arguments).unwrap(), // TODO: catch bad rendering (etc...missing variable)
                })))
            }
        }
    }
}

impl RenderingIterator {
    pub fn new(renderables: Vec<Renderable>, config: &TargetConfig) -> Result<Self, StencilError> {
        //let mut variables = HashMap::from([(
        //    "project_name".to_string(),
        //    TemplateVar::from(config.project.name.clone()),
        //)]);

        let arguments = serde_json::to_value(config)?;

        Ok(Self {
            renderables,
            arguments,
            index: 0,
        })
    }
}
