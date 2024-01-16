use super::Item;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Debug)]
pub struct Chapter {
    pub title: String,
    pub number: String,
    pub content: PathBuf,
    pub subchapters: Vec<Item>,
}

impl Chapter {
    pub fn new<P, S>(title: S, number: S, content: P, subchapters: Vec<Item>) -> Self
    where
        P: AsRef<Path>,
        S: Into<String>,
    {
        let title = title.into();
        let number = number.into();
        let content = content.as_ref().to_path_buf();

        Self {
            title,
            number,
            content,
            subchapters,
        }
    }
}
