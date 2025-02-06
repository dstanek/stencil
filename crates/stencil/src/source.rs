// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

mod config;
mod factory;
mod filesystem;
mod git;
mod model;

// public interface
#[allow(unused_imports)]
pub use factory::renderables;

#[allow(unused_imports)]
pub use model::{Directory, File, Renderable};
