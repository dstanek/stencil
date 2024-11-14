use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};

use crate::error::StencilError;
use crate::source::model::{Directory, File, Renderable};

//pub struct FilesystemCrawler {
//    root: PathBuf,
//}

//impl FilesystemCrawler {
//    pub fn new<P: AsRef<Path>>(root: P) -> Self {
//        FilesystemCrawler {
//            root: root.as_ref().to_path_buf(),
//        }
//    }
//
//    pub fn crawl(&self) -> Result<FilesystemIterator, StencilError> {
//        FilesystemIterator::new(&self.root)
//    }
//}

pub struct FilesystemIterator {
    stack: Vec<ReadDir>,
    root: PathBuf,
}

impl FilesystemIterator {
    pub fn new(root: &PathBuf) -> Result<Self, StencilError> {
        let stack = vec![fs::read_dir(root)?];
        Ok(FilesystemIterator {
            stack,
            root: root.clone(),
        })
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
                        //println!("X File: {:?}", path);
                        //println!("X Relative path: {:?}", relative_path);
                        return Some(Ok(Renderable::File(File::new(relative_path, &path))));
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
