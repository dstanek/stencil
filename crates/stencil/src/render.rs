// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use std::path::PathBuf;

use liquid::ParserBuilder;
use liquid::{object, Object};

use crate::target_config::TargetConfig;
use stencil_error::StencilError;
use stencil_source::{Directory, File, Renderable};

pub struct RenderingIterator {
    renderables: Vec<Renderable>,
    globals: Object,
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

        // TODO: get rid of the unwraps so that errors can be handled properly
        let parser = ParserBuilder::with_stdlib().build().unwrap();
        match renderable {
            Renderable::File(file) => {
                let template = parser.parse(&file.relative_path).unwrap();
                let relative_path = template.render(&self.globals).unwrap();
                let mut xrelative_path = PathBuf::from(&file.relative_path);

                if let Some(extention) = xrelative_path.extension() {
                    if extention != "liquid" {
                        return Some(Ok(Renderable::File(File {
                            relative_path,
                            content: file.content.clone(), // TODO: can i get rid of this clone?
                        })));
                    }
                }

                let template = parser.parse(file.content.as_str()).unwrap();
                let content = template.render(&self.globals).unwrap();

                xrelative_path.set_extension("");
                Some(Ok(Renderable::File(File {
                    relative_path: xrelative_path.to_string_lossy().into_owned(),
                    content,
                })))
            }
            Renderable::Directory(directory) => {
                let template = parser.parse(&directory.relative_path).unwrap();
                let path = template.render(&self.globals).unwrap();
                Some(Ok(Renderable::Directory(Directory {
                    relative_path: path,
                })))
            }
        }
    }
}

impl RenderingIterator {
    pub fn new(renderables: Vec<Renderable>, config: &TargetConfig) -> Self {
        let globals = object!({
            "project_name": config.project.name,
        });
        Self {
            renderables,
            globals,
            index: 0,
        }
    }
}
