use super::{Summarizer, Summary};
use crate::util;
use crate::{Chapter, Item};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_CHAPTER_FILE_NAMES: [&str; 4] = ["index", "readme", "INDEX", "README"];

/// It creates a summary using the file tree. The ordering is random. It depends in what it finds first.
/// It supports chapters and subchapters, but not sections. Each directory is a chapter and it must
/// contain a file named "index.md", "readme.md", "INDEX.md", "README.md" or a file with the same name as the directory.
/// Any other files are considered as subchapters. Standalone files are also considered main chapters.
///
/// # Example
///
/// A file tree such as this:
///
/// ├── chapter1/
/// │   ├── index.md
/// │   ├── subchapter1
/// │   └── subchapter2
/// ├── chapter2.md
/// └── chapter3.md
///
/// Will be summarized as:
///
/// Chapter1 (chapter1/index.md) (1)
/// ├── Subchapter1 (chapter1/subchapter1.md) (1.1)
/// └── Subchapter2 (chapter1/subchapter2.md) (1.2)
/// Chapter2 (chapter2.md) (2)
/// Chapter3 (chapter3.md) (3)
pub struct FileTreeSummarizer {
    path: PathBuf,
}

impl FileTreeSummarizer {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// It finds all the chapters in [`self.path`] recursively. It takes an initial chapter number
    /// where the enumeration will start.
    fn find_chapters<S>(&self, initial_chapter_number: S) -> Result<Vec<Chapter>>
    where
        S: ToString,
    {
        let dir_entries = fs::read_dir(&self.path)
            .with_context(|| anyhow!("Failed to read contentes of {}", self.path.display()))?;
        let mut chapter_number: String = initial_chapter_number.to_string();

        Ok(dir_entries
            .filter_map(|entry| {
                let entry = entry.ok()?;

                if entry.file_type().ok()?.is_dir() {
                    let chapter = Chapter::new(
                        self.format_chapter_title(entry.path()),
                        chapter_number.clone(),
                        self.find_main_chapter_content(entry.path()).ok()?,
                        FileTreeSummarizer::new(entry.path())
                            .find_chapters(chapter_number.clone() + ".1")
                            .ok()?,
                    );

                    return Some(chapter);
                }

                if self.is_parent_content(&entry.path()) {
                    return None;
                }

                let chapter = Chapter::new(
                    self.format_chapter_title(entry.path()),
                    chapter_number.clone(),
                    entry.path(),
                    Vec::new(),
                );
                chapter_number = util::next_chapter_number(&chapter_number);

                Some(chapter)
            })
            .collect::<Vec<Chapter>>())
    }

    /// It returns a formatted chapter title for the given file name. It capitalizes the first letter and removes the extension.
    ///
    /// # Example
    ///
    /// chapter.md -> Chapter
    /// chapter2.md -> Chapter2
    fn format_chapter_title(&self, file_name: PathBuf) -> String {
        let file_name = file_name.file_stem().unwrap().to_string_lossy();
        let mut file_name_iter = file_name.chars();

        match file_name_iter.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + file_name_iter.as_str(),
        }
    }

    /// It returns the content (a path) for the given main chapter (a directory). It looks for multiple files:
    ///
    /// - index
    /// - readme
    /// - INDEX
    /// - README
    /// - A file with the same name as the directory
    ///
    /// If none are found it returns an error.
    fn find_main_chapter_content<P>(&self, path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;

            if self.is_parent_content(Path::new(&entry.path())) {
                return Ok(entry.path());
            }
        }

        anyhow::bail!(
            "Could not find content for chapter {}. Create a index or a summary.",
            path.as_ref().display()
        )
    }

    // Is it safe to unwrap here?
    fn is_parent_content(&self, path: &Path) -> bool {
        for supported_name in SUPPORTED_CHAPTER_FILE_NAMES {
            if path.file_stem().unwrap().to_string_lossy() == supported_name {
                return true;
            }
        }

        path.parent().unwrap_or(Path::new("")).file_name().unwrap() == path.file_stem().unwrap()
    }
}

impl Summarizer for FileTreeSummarizer {
    fn summarize(&self) -> Result<Summary> {
        let items = self
            .find_chapters("1")?
            .iter()
            .map(|chapter| return Item::from(chapter.clone()))
            .collect();

        Ok(Summary::new(items))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn it_should_summarize_nested_chapters() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let chapter_path = temp_dir.path().join("chapter1");
        let expected = Chapter::new(
            "Chapter1",
            "1",
            chapter_path.join("chapter1.md"),
            vec![
                Chapter::new(
                    "Chapter1.1",
                    "1.1",
                    chapter_path.join("chapter1.1.md"),
                    Vec::new(),
                ),
                Chapter::new(
                    "Chapter1.2",
                    "1.2",
                    chapter_path.join("chapter1.2.md"),
                    Vec::new(),
                ),
                Chapter::new(
                    "Chapter1.3",
                    "1.3",
                    chapter_path.join("chapter1.3.md"),
                    Vec::new(),
                ),
            ],
        );

        fs::create_dir(&chapter_path)?;
        fs::write(&chapter_path.join("chapter1.md"), "")?;
        for subchapter in expected.subchapters.iter() {
            fs::write(temp_dir.path().join(&subchapter.content), "")?;
        }

        assert_eq!(
            vec![expected],
            FileTreeSummarizer::new(temp_dir.path()).find_chapters("1")?
        );

        Ok(())
    }

    #[test]
    fn it_should_summarize_non_nested_chapters() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;

        let expected = vec![
            Item::from(Chapter::new(
                "Chapter1",
                "1",
                temp_dir.path().join("chapter1.md"),
                Vec::new(),
            )),
            Item::from(Chapter::new(
                "Chapter2",
                "2",
                temp_dir.path().join("chapter2.md"),
                Vec::new(),
            )),
            Item::from(Chapter::new(
                "Chapter3",
                "3",
                temp_dir.path().join("chapter3"),
                Vec::new(),
            )),
            Item::from(Chapter::new(
                "Chapter4",
                "4",
                temp_dir.path().join("chapter4.txt"),
                Vec::new(),
            )),
        ];

        for chapter in expected.iter() {
            match chapter {
                Item::Chapter(chapter) => {
                    fs::write(temp_dir.path().join(&chapter.content), "")?;
                }
                _ => {}
            }
        }

        let summary = FileTreeSummarizer::new(temp_dir.path()).summarize()?;

        assert_eq!(expected, summary.items);

        Ok(())
    }

    #[test]
    fn it_should_format_the_file_name() -> Result<(), Box<dyn Error>> {
        let summarizer = FileTreeSummarizer::new("");
        let tests = vec![
            ("chapter.md", "Chapter"),
            ("intro", "Intro"),
            ("file.txt", "File"),
        ];

        for test in tests.iter() {
            assert_eq!(
                summarizer.format_chapter_title(PathBuf::from(test.0)),
                test.1
            );
        }

        Ok(())
    }

    // If it works with "index.md" works with the other ones. Right? It works with the other ones.
    // RIGHT?
    #[test]
    fn it_should_return_the_content_for_the_given_main_chapter() -> Result<(), Box<dyn Error>> {
        let summarizer = FileTreeSummarizer::new("");
        let temp_dir = tempdir()?;
        let main_chapter_path = temp_dir.path().join("chapter1");

        fs::create_dir(&main_chapter_path)?;
        fs::write(&main_chapter_path.join("index.md"), "")?;

        let content = summarizer.find_main_chapter_content(&main_chapter_path)?;

        assert_eq!(content, main_chapter_path.join("index.md"));

        Ok(())
    }

    #[test]
    fn it_should_return_the_content_for_the_given_main_chapter_when_it_has_the_dir_name(
    ) -> Result<(), Box<dyn Error>> {
        let summarizer = FileTreeSummarizer::new("");
        let temp_dir = tempdir()?;
        let main_chapter_path = temp_dir.path().join("chapter1");

        fs::create_dir(&main_chapter_path)?;
        fs::write(&main_chapter_path.join("chapter1.md"), "")?;

        let content = summarizer.find_main_chapter_content(&main_chapter_path)?;

        assert_eq!(content, main_chapter_path.join("chapter1.md"));

        Ok(())
    }
}
