// Copyright (c) 2025 David Stanek <dstanek@dstanek.com>

use stencil_rendering::Renderable;

pub struct File {
    pub content: String,
}

impl File {
    pub fn new(content: &str) -> Self {
        File {
            content: content.to_string(),
        }
    }
}

impl Renderable for File {
    fn content(&self) -> &str {
        &self.content
    }
}
