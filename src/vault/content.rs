mod chapter;
mod section;

use super::summary::{FileTreeSummarizer, Summary};
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

pub struct Content {
    pub chapters: Vec<Chapter>,
    pub sections: Vec<Section>,
    pub summary: Summary,
}

impl Content {
    pub fn new<P>(path: P) -> Content
    where
        P: AsRef<Path>,
    {
        let summary = Content::get_summary(path);
        let sections = Content::get_sections();
        let chapters = Content::get_chapters();

        Content {
            chapters,
            sections,
            summary,
        }
    }

    // Just iterate over the summary and filter
    fn get_chapters() -> Vec<Chapter> {
        unimplemented!()
    }

    // Just iterate over the summary and filter
    fn get_sections() -> Vec<Section> {
        unimplemented!()
    }

    fn get_summary<P>(path: P) -> Summary
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let has_summary_file = path.join("summary.md").exists();

        if has_summary_file {
            unimplemented!();
        } else {
            let summarizer = Box::new(FileTreeSummarizer::new(path));
            Summary::new(summarizer)
        }
    }
}
