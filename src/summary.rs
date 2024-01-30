mod file_tree_summarizer;

pub use file_tree_summarizer::FileTreeSummarizer;

use anyhow::Result;

use crate::Item;

pub trait Summarizer {
    fn summarize(&self) -> Result<Vec<Item>>;
}

pub struct Summary {
    pub summarizer: Box<dyn Summarizer>,
}

impl Summary {
    pub fn new(summarizer: Box<dyn Summarizer>) -> Summary {
        Summary { summarizer }
    }

    pub fn summarize(&mut self) -> Result<Vec<Item>> {
        Ok(self.summarizer.summarize()?)
    }
}
