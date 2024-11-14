use std::path::PathBuf;

use liquid::ParserBuilder;
use liquid::{object, Object};

use crate::error::StencilError;
use crate::source::filesystem::FilesystemIterator;
use crate::source::model::{Directory, File, Renderable};
use crate::target_config::TargetConfig;

pub struct RenderingIterator {
    pub iterator: FilesystemIterator,
    pub globals: Object,
}

impl Iterator for RenderingIterator {
    type Item = Result<Renderable, StencilError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(renderable) = self.iterator.next() {
            match renderable {
                Ok(Renderable::File(file)) => {
                    let parser = ParserBuilder::with_stdlib().build().unwrap();
                    let template = parser.parse(file.relative_path.to_str()?).unwrap();
                    let relative_path = template.render(&self.globals).unwrap();
                    let mut relative_path = PathBuf::from(relative_path);

                    if let Some(extention) = file.relative_path.extension() {
                        if extention != "liquid" {
                            return Some(Ok(Renderable::File(File {
                                relative_path,
                                content: file.content,
                            })));
                        }
                    }

                    let template = parser.parse(file.content.as_str()).unwrap();
                    let content = template.render(&self.globals).unwrap();

                    relative_path.set_extension("");
                    // println!("relative_path: {:?}", relative_path);
                    return Some(Ok(Renderable::File(File {
                        relative_path,
                        content,
                    })));
                }
                Ok(Renderable::Directory(directory)) => {
                    let parser = ParserBuilder::with_stdlib().build().unwrap();
                    let template = parser.parse(directory.relative_path.to_str()?).unwrap();
                    let path = template.render(&self.globals).unwrap();
                    // println!("Rendered directory: {:?}", path);
                    let directory = Directory {
                        relative_path: PathBuf::from(path),
                    };
                    return Some(Ok(Renderable::Directory(directory)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}

impl RenderingIterator {
    pub fn new(iterator: FilesystemIterator, config: &TargetConfig) -> RenderingIterator {
        let globals = object!({
            "project_name": config.project.name,
        });
        RenderingIterator { iterator, globals }
    }
}
