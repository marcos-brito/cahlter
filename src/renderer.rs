mod askama_renderer;

use crate::config::Config;
use crate::Chapter;
use crate::Content;
use anyhow::Result;
pub use askama_renderer::AskamaRenderer;
use std::path::PathBuf;

pub trait Renderer {
    fn render(&self, chapter: &Chapter) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct RendererContext {
    content: Content,
    config: Config,
    // src_dir so we can strip from the chapter content and get a proper url.
    src_dir: PathBuf,
}

impl RendererContext {
    pub fn new(content: Content, config: Config, src_dir: PathBuf) -> Self {
        Self {
            content,
            config,
            src_dir,
        }
    }
}
