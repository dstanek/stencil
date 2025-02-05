use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};

use crate::error::StencilError;
use crate::source::model::{Directory, File, Renderable, RenderableIterator};

pub struct FilesystemIterator {
    stack: Vec<ReadDir>,
    root: PathBuf,
}

impl FilesystemIterator {
    pub fn new(root: &PathBuf) -> Result<Box<dyn RenderableIterator>, StencilError> {
        let stack = vec![fs::read_dir(root)?];
        let iterator = FilesystemIterator {
            stack,
            root: root.clone(),
        };
        Ok(Box::new(iterator))
    }
}

impl Iterator for FilesystemIterator {
    type Item = Result<Renderable, StencilError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(dir) = self.stack.last_mut() {
            match dir.next() {
                Some(Ok(entry)) => {
                    let path = entry.path();
                    let relative_path = path.strip_prefix(&self.root).unwrap().to_path_buf();
                    if path.is_dir() {
                        self.stack.push(fs::read_dir(&path).unwrap());
                        return Some(Ok(Renderable::Directory(Directory::new(relative_path))));
                    } else {
                        return Some(Ok(Renderable::File(
                            File::from_path(relative_path, &path).unwrap(),
                        )));
                    }
                }
                Some(Err(e)) => return Some(Err(StencilError::Other(e.to_string()))),
                None => {
                    self.stack.pop();
                }
            }
        }
        None
    }
}
