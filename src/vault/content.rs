mod chapter;
mod section;

use crate::summary::{FileTreeSummarizer, Summarizer, Summary};
use anyhow::{Context, Result};
pub use chapter::Chapter;
pub use section::Section;
use std::convert::From;
use std::path::Path;

#[derive(Clone, PartialEq, Debug)]
pub enum Item {
    Chapter(Chapter),
    Section(Section),
}

impl From<Chapter> for Item {
    fn from(chapter: Chapter) -> Self {
        Item::Chapter(chapter)
    }
}

impl From<Section> for Item {
    fn from(section: Section) -> Self {
        Item::Section(section)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Content {
    pub summary: Summary,
}

impl Content {
    pub fn new<P>(path: P) -> Result<Content>
    where
        P: AsRef<Path>,
    {
        let summary = Content::create_summary(path)?;

        Ok(Content { summary })
    }

    // Just iterate over the summary and filter
    pub fn chapters(&self) -> Vec<Chapter> {
        self.summary
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Chapter(chapter) => Some(chapter.clone()),
                _ => None,
            })
            .collect()
    }

    // Just iterate over the summary and filter
    pub fn sections(&self) -> Vec<Section> {
        self.summary
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Section(section) => Some(section.clone()),
                _ => None,
            })
            .collect()
    }

    fn create_summary<P>(path: P) -> Result<Summary>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let has_summary_file = path.join("summary.md").exists();

        if has_summary_file {
            unimplemented!();
        }

        let summarizer = FileTreeSummarizer::new(&path);

        Ok(summarizer
            .summarize()
            .with_context(|| format!("Could not summarize the content in {}", &path.display()))?)
    }
}
