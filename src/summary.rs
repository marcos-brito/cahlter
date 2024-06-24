mod file_tree_summarizer;

pub use file_tree_summarizer::FileTreeSummarizer;

use anyhow::Result;

use crate::Item;

pub trait Summarizer {
    fn summarize(&self) -> Result<Summary>;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Summary {
    pub items: Vec<Item>,
}

impl Summary {
    pub fn new(items: Vec<Item>) -> Summary {
        Summary { items }
    }
}
