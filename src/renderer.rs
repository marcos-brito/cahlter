mod askama_renderer;

pub use askama_renderer::AskamaRenderer;

use crate::config::Config;
use crate::Content;

use anyhow::Result;

pub trait Renderer {
    fn render(&self) -> Result<Vec<String>>;
}

pub struct Context {
    pub content: Content,
    pub config: Config,
}

impl Context {
    pub fn new(content: Content, config: Config) -> Self {
        Self { content, config }
    }
}
