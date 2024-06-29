use super::{Summarizer, Summary};
use crate::util;
use crate::{Chapter, Item};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_CHAPTER_FILE_NAMES: [&str; 4] = ["index.md", "readme.md", "INDEX.md", "README.md"];

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
        S: Into<String>,
    {
        let dir_entries = fs::read_dir(&self.path)?;
        let mut summary = Vec::new();
        let mut chapter_number: String = initial_chapter_number.into();

        for entry in dir_entries {
            let entry = entry?;
            let entry_type = entry.file_type()?;

            if FileTreeSummarizer::is_md_file(&entry)?
                && !FileTreeSummarizer::is_parent_content(&entry)
            {
                let chapter = Chapter::new(
                    FileTreeSummarizer::format_chapter_title(
                        entry.file_name().into_string().unwrap(),
                    ),
                    chapter_number.clone(),
                    entry.path(),
                    Vec::new(),
                );

                summary.push(chapter);
                chapter_number = util::next_chapter_number(chapter_number);
                continue;
            }

            if entry_type.is_dir() {
                let subchapters = FileTreeSummarizer::new(entry.path())
                    .find_chapters(format!("{}.1", chapter_number.clone()))?;

                let chapter = Chapter::new(
                    FileTreeSummarizer::format_chapter_title(
                        entry.file_name().into_string().unwrap(),
                    ),
                    chapter_number.clone(),
                    FileTreeSummarizer::find_main_chapter_content(entry.path())?,
                    subchapters,
                );

                summary.push(chapter);
            }
        }

        Ok(summary)
    }

    /// It returns a formatted chapter title for the given file name. It capitalizes the first letter and removes the extension.
    ///
    /// # Example
    ///
    /// chapter.md -> Chapter
    /// chapter2.md -> Chapter2
    fn format_chapter_title(file_name: String) -> String {
        let file_name = file_name.replace(".md", "");
        let mut file_name_iter = file_name.chars();

        match file_name_iter.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + file_name_iter.as_str(),
        }
    }

    /// It returns the content (a path) for the given main chapter (a directory). It looks for multiple files:
    ///
    /// - index.md
    /// - readme.md
    /// - INDEX.md
    /// - README.md
    /// - A file with the same name as the directory
    ///
    /// If none are found it returns an error.
    fn find_main_chapter_content<P>(path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();

        for file_name in SUPPORTED_CHAPTER_FILE_NAMES.iter() {
            let file_path = path.join(file_name);

            if file_path.exists() {
                return Ok(file_path);
            }
        }

        let chapter_name = path.file_name().unwrap().to_str().unwrap().to_string() + ".md";

        if path.join(&chapter_name).exists() {
            return Ok(path.join(&chapter_name));
        }

        Err(anyhow!(
            "Could not found content for chapter {}. Create a index.md or a summary.md.",
            path.display()
        ))
    }

    fn is_md_file(entry: &fs::DirEntry) -> Result<bool> {
        Ok(entry.file_type()?.is_file() && entry.path().extension().unwrap() == "md")
    }

    // My brain is dead. This might be shit coding, but I can't tell. I see the resemblance between
    // this and that other crap find_main_chapter_content and that is it.
    fn is_parent_content(entry: &fs::DirEntry) -> bool {
        let entry_path = entry.path();
        let entry_name = entry_path.file_name().unwrap().to_str().unwrap();

        for &file_name in SUPPORTED_CHAPTER_FILE_NAMES.iter() {
            if entry_name == file_name {
                return true;
            }
        }

        let chapter_name = entry_path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + ".md";

        if chapter_name == entry_name {
            return true;
        }

        false
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
        let main_chapter_path = temp_dir.path().join("chapter1");

        let subchapters = vec![
            Chapter::new(
                "Chapter1.1",
                "1.1",
                main_chapter_path.join("chapter1.1.md"),
                Vec::new(),
            ),
            Chapter::new(
                "Chapter1.2",
                "1.2",
                main_chapter_path.join("chapter1.2.md"),
                Vec::new(),
            ),
            Chapter::new(
                "Chapter1.3",
                "1.3",
                main_chapter_path.join("chapter1.3.md"),
                Vec::new(),
            ),
        ];

        let expected = Item::from(Chapter::new(
            "Chapter1",
            "1",
            main_chapter_path.join("chapter1.md"),
            subchapters.clone(),
        ));

        fs::create_dir(&main_chapter_path)?;

        for subchapter in subchapters.iter() {
            fs::write(temp_dir.path().join(&subchapter.content), "")?;
        }
        fs::write(&main_chapter_path.join("chapter1.md"), "")?;

        let summary = FileTreeSummarizer::new(temp_dir.path()).summarize()?;

        assert_eq!(vec![expected], summary.items);

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
                temp_dir.path().join("chapter3.md"),
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
        let file_name = String::from("chapter1.md");
        let processed_file_name = FileTreeSummarizer::format_chapter_title(file_name);

        assert_eq!(processed_file_name, String::from("Chapter1"));

        Ok(())
    }

    // If it works with "index.md" works with the other ones. Right? It works with the other ones.
    // RIGHT?
    #[test]
    fn it_should_return_the_content_for_the_given_main_chapter() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let main_chapter_path = temp_dir.path().join("chapter1");

        fs::create_dir(&main_chapter_path)?;
        fs::write(&main_chapter_path.join("index.md"), "")?;

        let content = FileTreeSummarizer::find_main_chapter_content(&main_chapter_path)?;

        assert_eq!(content, main_chapter_path.join("index.md"));

        Ok(())
    }

    #[test]
    fn it_should_return_the_content_for_the_given_main_chapter_when_it_has_the_dir_name(
    ) -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let main_chapter_path = temp_dir.path().join("chapter1");

        fs::create_dir(&main_chapter_path)?;
        fs::write(&main_chapter_path.join("chapter1.md"), "")?;

        let content = FileTreeSummarizer::find_main_chapter_content(&main_chapter_path)?;

        assert_eq!(content, main_chapter_path.join("chapter1.md"));

        Ok(())
    }
}
