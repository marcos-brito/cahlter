mod file_tree_summarizer;
mod summary_file;

use crate::Item;
use anyhow::Result;
pub use file_tree_summarizer::FileTreeSummarizer;
pub use summary_file::SummaryFileSummarizer;

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
