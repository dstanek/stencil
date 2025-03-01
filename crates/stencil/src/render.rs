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
                let path_template = parser.parse(&file.relative_path).unwrap();
                let mut dest_path = PathBuf::from(path_template.render(&self.globals).unwrap());

                if let Some(extention) = dest_path.extension() {
                    if extention != "liquid" {
                        return Some(Ok(Renderable::File(File {
                            relative_path: dest_path.to_string_lossy().to_string(),
                            content: file.content.clone(), // TODO: can i get rid of this clone?
                        })));
                    }
                }

                let template = parser.parse(file.content.as_str()).unwrap();
                let content = template.render(&self.globals).unwrap(); // TODO: catch bad rendering (etc...missing variable)

                dest_path.set_extension("");
                Some(Ok(Renderable::File(File {
                    relative_path: dest_path.to_string_lossy().to_string(),
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
        let mut globals = object!({
            "project_name": config.project.name,
        });

        for (key, value) in &config.arguments {
            globals.insert(
                key.clone().into(),
                liquid::model::Value::scalar(value.clone()),
            );
        }

        Self {
            renderables,
            globals,
            index: 0,
        }
    }
}
