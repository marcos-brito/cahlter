mod file_tree_summarizer;

pub use file_tree_summarizer::FileTreeSummarizer;

use super::content::Item;
use anyhow::Result;

pub trait Summarizer {
    fn summarize(&self) -> Result<Vec<Item>>;
}

pub struct Summary {
    pub summary: Vec<Item>,
    pub summarizer: Box<dyn Summarizer>,
}

impl Summary {
    pub fn new(summarizer: Box<dyn Summarizer>) -> Summary {
        Summary {
            summary: Vec::new(),
            summarizer,
        }
    }

    fn summarize(&mut self) -> Result<()> {
        self.summary = self.summarizer.summarize()?;
        Ok(())
    }
}
