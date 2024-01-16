use super::chapter::Chapter;

#[derive(Clone, PartialEq, Debug)]
pub struct Section {
    pub title: String,
    pub chapter: Vec<Chapter>,
}

impl Section {
    pub fn new(title: String, chapter: Vec<Chapter>) -> Self {
        Self { title, chapter }
    }
}
