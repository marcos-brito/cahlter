mod askama_renderer;

use crate::config::Config;
use crate::Chapter;
use crate::Content;
use anyhow::Result;
pub use askama_renderer::AskamaRenderer;

pub trait Renderer {
    fn render(&self, chapter: &Chapter) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct RendererContext {
    pub content: Content,
    pub config: Config,
}

impl RendererContext {
    pub fn new(content: Content, config: Config) -> Self {
        Self { content, config }
    }
}
