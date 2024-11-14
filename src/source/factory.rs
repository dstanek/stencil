use std::path::PathBuf;

use crate::error::StencilError;
//use crate::source::filesystem::FilesystemCrawler;
use crate::source::filesystem::FilesystemIterator;
use crate::source::model::Renderable;

//pub fn renderables(source: String) -> Result<Renderable> {
//pub fn renderables(source: &String) -> Result<FilesystemIterator, StencilError> {
pub fn renderables(source: &String) -> Result<FilesystemIterator, StencilError> {
    let stencil_path = PathBuf::from(source);
    //pub fn renderables(source: &String) -> Result<Vec<Renderable>, StencilError> {
    //let crawler = FilesystemCrawler::new(source);
    let iterator = FilesystemIterator::new(&stencil_path)?;
    Ok(iterator)
    //let iterator = crawler.crawl()?;
    //    let renderables: Vec<Renderable> = iterator.filter_map(Result::ok).collect();
    //    Ok(renderables)
}
